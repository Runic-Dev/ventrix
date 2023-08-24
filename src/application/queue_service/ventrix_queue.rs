use std::{error::Error, sync::Arc};

use actix_web::web;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::{
    common::types::VentrixEvent,
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

    pub fn start_event_processor(&self, receiver: Receiver<VentrixEvent>) {
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
                Ok(listening_services) => {
                    for service in listening_services {
                        let body = VentrixEvent {
                            id: event.id,
                            event_type: event.event_type.clone(),
                            payload: event.payload.clone(),
                        };

                        {
                            let response = client_lock
                                .post(service.endpoint.clone())
                                .json::<VentrixEvent>(&body)
                                .send()
                                .await;
                            if response.is_ok() {
                                match database.fulfil_event(&event).await {
                                    Ok(_) => {
                                        tracing::info!(
                                            "Event {} was sent to Service {} successfully",
                                            event.event_type,
                                            service.name
                                        );
                                    }
                                    Err(_) => {
                                        tracing::info!(
                                            "Event {} was sent to Service {} successfully, but was not able to update the database",
                                            event.event_type,
                                            service.name
                                        );
                                    }
                                }
                            } else {
                                tracing::warn!(
                                    "Event {} failed to send to Service {}",
                                    event.event_type,
                                    service.name
                                );
                            }
                        };
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
        service_name: &str,
        event_type_name: &str,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        self.database
            .get_ref()
            .register_service_for_event_type(service_name, event_type_name)
            .await
    }
}
