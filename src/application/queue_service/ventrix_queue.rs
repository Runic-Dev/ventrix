use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::{common::types::VentrixEvent, domain::models::service::Service};

use super::{EventServiceMap, ListenToEventResult};

#[derive(Debug)]
pub struct VentrixQueue {
    pub sender: Sender<VentrixEvent>,
    event_type_to_services: Arc<Mutex<EventServiceMap>>,
}

impl VentrixQueue {
    pub async fn new(sender: Sender<VentrixEvent>) -> Self {
        Self {
            sender,
            event_type_to_services: Arc::new(Mutex::new(
                HashMap::<String, HashSet<Service>>::default(),
            )),
        }
    }

    pub fn start_event_processor(&self, receiver: Receiver<VentrixEvent>) {
        let event_map = Arc::clone(&self.event_type_to_services);

        tokio::spawn(async move {
            Self::event_processor(receiver, event_map).await;
        });
    }

    async fn event_processor(
        mut receiver: Receiver<VentrixEvent>,
        event_map: Arc<Mutex<HashMap<String, HashSet<Service>>>>,
    ) {
        while let Some(event) = receiver.recv().await {
            tracing::info!("Processing event: {}", &event.event_type);
            let locked_event_service_map = event_map.lock().await;
            if let Some(listening_services) =
                locked_event_service_map.get::<String>(&event.event_type)
            {
                let client = reqwest::Client::new();
                for service in listening_services {
                    let body = VentrixEvent {
                        event_type: event.event_type.clone(),
                        payload: event.payload.clone(),
                    };
                    let response = client
                        .post(service.endpoint.clone())
                        .json::<VentrixEvent>(&body)
                        .send()
                        .await;

                    if response.is_ok() {
                        tracing::info!(
                            "Event {} was sent to Service {} successfully",
                            event.event_type,
                            service.name
                        )
                    } else {
                        tracing::warn!(
                            "Event {} failed to send to Service {}",
                            event.event_type,
                            service.name
                        )
                    }
                }
            }
        }
    }

    pub async fn listen_to_event(
        &self,
        service: &Service,
        event_type: &String,
    ) -> ListenToEventResult {
        let mut event_map_lock = self.event_type_to_services.lock().await;

        if let Some(registered_services) = event_map_lock.get_mut(event_type) {
            registered_services.insert(service.clone());
            ListenToEventResult::Existed
        } else {
            let mut registered_services = HashSet::<Service>::new();
            registered_services.insert(service.clone());
            event_map_lock.insert(event_type.clone(), registered_services);
            ListenToEventResult::NewEntry
        }
    }
}
