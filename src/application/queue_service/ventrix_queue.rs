use std::{error::Error, sync::Arc};

use actix_web::web;
use chrono::{Duration, Utc};
use reqwest::Client;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::{
    common::types::{EventFulfillmentDetails, ListenToEventReq, VentrixEvent},
    infrastructure::persistence::{Database, InsertDataResponse},
};

#[derive(Debug)]
pub struct VentrixQueue {
    pub sender: Sender<VentrixEvent>,
    database: web::Data<dyn Database>,
}

impl VentrixQueue {
    pub async fn new(database: web::Data<dyn Database>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel::<VentrixEvent>(50);
        let ventrix_queue = Self {
            sender: sender.clone(),
            database,
        };
        ventrix_queue.start_event_processor(receiver, sender);
        ventrix_queue
    }

    async fn event_processor(
        mut receiver: Receiver<VentrixEvent>,
        database: web::Data<dyn Database>,
    ) {
        let client = Arc::new(Mutex::new(reqwest::Client::new()));
        let database = database.get_ref();

        while let Some(event) = receiver.recv().await {
            tracing::info!("Processing event: {}", &event.event_type);

            match database.get_service_by_event_type(&event.event_type).await {
                Ok(details_for_listening_services) => {
                    Self::send_to_listening_services(
                        details_for_listening_services,
                        event,
                        Arc::clone(&client),
                        database,
                    )
                    .await
                }
                Err(_) => {
                    tracing::error!(
                        "Failed to find any services registered to listen to event {}",
                        event.event_type
                    );
                }
            }
        }
    }

    pub async fn listen_to_event(
        &self,
        listen_to_event_req: &ListenToEventReq,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        self.database
            .get_ref()
            .register_service_for_event_type(listen_to_event_req)
            .await
    }

    pub async fn publish_event(
        &self,
        event: VentrixEvent,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<VentrixEvent>> {
        self.sender.send(event.clone()).await
    }

    fn start_event_processor(
        &self,
        receiver: Receiver<VentrixEvent>,
        failed_event_sender: Sender<VentrixEvent>,
    ) {
        let event_processor_db = web::Data::clone(&self.database);
        tokio::spawn(async move {
            Self::event_processor(receiver, event_processor_db).await;
        });
        let failed_events_process_db = web::Data::clone(&self.database);
        tokio::spawn(async move {
            loop {
                if let Ok(failed_events) = failed_events_process_db.get_failed_events().await.map_err(|err| {
                tracing::warn!("There was an issue fetching a list of failed events from the database: {}", err);
            }) {
                let count = failed_events.len();
                let failed_events_iter = failed_events.into_iter();
                tracing::info!("Retreived {} failed events from database", count);
                for failed_event in failed_events_iter {
                    match failed_event_sender.send(failed_event.clone()).await {
                        Ok(_) => tracing::info!("Failed event {} published to queue", failed_event.event_type),
                        Err(err) => tracing::warn!("Unable to send events to inner channel. Error: {}", err)
                    }
                }
            };
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
        });
    }

    async fn send_to_listening_services(
        details_for_listening_services: Vec<EventFulfillmentDetails>,
        mut event: VentrixEvent,
        client: Arc<Mutex<Client>>,
        database: &dyn Database,
    ) {
        let client_lock = client.lock().await;
        for fulfillment_details in details_for_listening_services {
            let body = event.clone();

            let destination = format!(
                "{}{}",
                fulfillment_details.url, fulfillment_details.endpoint
            );

            let response = client_lock
                .post(destination)
                .json::<VentrixEvent>(&body)
                .send()
                .await;

            match response {
                Ok(response_details) => match response_details.error_for_status() {
                    Ok(_) => Self::on_success_response(&event, database, fulfillment_details).await,
                    Err(server_error) => {
                        tracing::warn!("Event {} was not sent to Service {} - endpoint {} successfully. Error from server: {}",
                                    event.event_type, fulfillment_details.name, fulfillment_details.endpoint, server_error);

                        if let Some(details) = event.retry_details.as_mut() {
                            details.retry_count += 1;
                            let minutes_to_wait: i64 =
                                (details.retry_count + 1).try_into().unwrap_or(2);
                            details.retry_time = Utc::now() + Duration::minutes(minutes_to_wait);
                            match database
                                .update_retry_time(event.id, details.retry_time, details.retry_count)
                                .await
                            {
                                Ok(_) => {
                                    tracing::info!(
                                        "Retry details for event {} successfully updated",
                                        event.event_type
                                    )
                                }
                                Err(err) => {
                                    tracing::warn!(
                                        "Failed to persist retry details for event {}. Err: {}",
                                        event.event_type,
                                        err
                                    )
                                }
                            }

                            return;
                        }

                        match database.add_failed_event(&event).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Failed event {} was added to the failed_events table",
                                    &event.event_type
                                )
                            }
                            Err(err) => {
                                tracing::warn!(
                                    "Could not add failed event {} to failed_events table. Err: {}",
                                    &event.event_type,
                                    err
                                )
                            }
                        };
                    }
                },
                Err(err) => {
                    tracing::error!("Could not retrieve response from server: {}", err)
                }
            }
        }
    }

    async fn on_success_response(
        event: &VentrixEvent,
        database: &dyn Database,
        fulfillment_details: EventFulfillmentDetails,
    ) {
        if event.retry_details.is_some() {
            match database.resolve_failed_event(event.id).await {
                Ok(_) => {
                    tracing::info!(
                        "Failed event {} was sent to Service {} successfully",
                        event.event_type,
                        fulfillment_details.name
                    );
                }
                Err(err) => {
                    tracing::warn!(
                    "Failed event {} was sent to Service {} successfully, but was not able to update the failed events table. Error: {}",
                    event.event_type,
                    fulfillment_details.name,
                    err
                );
                }
            }
        }

        match database.fulfil_event(event).await {
            Ok(_) => {
                tracing::info!(
                    "Event {} was sent to Service {} successfully",
                    event.event_type,
                    fulfillment_details.name
                );
            }
            Err(_) => {
                tracing::info!(
                    "Event {} was sent to Service {} successfully, but was not able to update the database",
                    event.event_type,
                    fulfillment_details.name
                );
            }
        }
    }
}
