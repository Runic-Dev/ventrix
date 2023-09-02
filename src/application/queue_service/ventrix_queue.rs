use std::{error::Error, sync::Arc};

use actix_web::web;
use reqwest::Client;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::{
    common::types::{
        EventFulfillmentDetails, ListenToEventReq, VentrixEvent, VentrixQueueResponse,
    },
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
        let ventrix_queue = Self { sender, database };
        ventrix_queue.start_event_processor(receiver);
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

    pub async fn publish_event(&self, event: VentrixEvent) -> VentrixQueueResponse {
        match self.sender.send(event.clone()).await {
            Ok(_) => match self.database.save_published_event(&event).await {
                Ok(_) => {
                    VentrixQueueResponse::PublishedAndSaved(String::from("The event was successfully processed."))
                }
                Err(err) => {
                    VentrixQueueResponse::PublishedNotSaved(
                        format!("The event was published to the queue but was not successfully recorded to the database. Error message: {}", err)
                    )
                }
            },
            Err(err) => {
                // TODO: This means the channel is closed. Consider a recovery strategy.
                panic!("There is a fatal error within the Ventrix Queue: {}", err)
            }
        }
    }

    fn start_event_processor(&self, receiver: Receiver<VentrixEvent>) {
        let event_processor_db = web::Data::clone(&self.database);
        tokio::spawn(async move {
            Self::event_processor(receiver, event_processor_db).await;
        });
    }

    async fn send_to_listening_services(
        details_for_listening_services: Vec<EventFulfillmentDetails>,
        event: VentrixEvent,
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

                        if event.is_retry {
                            // TODO: Update retry_count and retry_time
                            return;
                        } 

                        match database.add_failed_event(&event).await {
                            Ok(_) => {
                                tracing::info!("Failed event {} was added to the failed_events table",
                                    &event.event_type)
                            },
                            Err(err) => {
                                tracing::warn!("Could not add failed event {} to failed_events table. Err: {}",
                                    &event.event_type,
                                    err)
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
        if event.is_retry {
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
                    )
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
