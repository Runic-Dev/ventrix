use std::{error::Error, sync::Arc};

use actix_web::web;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::{
    common::types::{ListenToEventReq, VentrixEvent, VentrixQueueResponse},
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

    fn start_event_processor(&self, receiver: Receiver<VentrixEvent>) {
        let event_processor_db = web::Data::clone(&self.database);
        tokio::spawn(async move {
            Self::event_processor(receiver, event_processor_db).await;
        });
    }

    async fn event_processor(
        mut receiver: Receiver<VentrixEvent>,
        database: web::Data<dyn Database>,
    ) {
        let client = Arc::new(Mutex::new(reqwest::Client::new()));
        let client_lock = client.lock().await;
        let database = database.get_ref();

        while let Some(event) = receiver.recv().await {
            tracing::info!("Processing event: {}", &event.event_type);

            match database.get_service_by_event_type(&event.event_type).await {
                Ok(details_for_listening_services) => {
                    for fulfillment_details in details_for_listening_services {
                        let body = VentrixEvent {
                            id: event.id,
                            event_type: event.event_type.clone(),
                            payload: event.payload.clone(),
                        };

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
                                Ok(_) => match database.fulfil_event(&event).await {
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
                                },
                                Err(server_error) => {
                                    tracing::warn!("Event {} was not sent to Service {} - endpoint {} successfully. Error from server: {}",
                                    event.event_type, fulfillment_details.name, fulfillment_details.endpoint, server_error)
                                }
                            },
                            Err(err) => {
                                tracing::error!("Could not retrieve response from server: {}", err)
                            }
                        }
                    }
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
}
