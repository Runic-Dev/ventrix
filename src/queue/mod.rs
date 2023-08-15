use crate::{models::service::Service, types::VentrixEvent};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use reqwest;

type UniqueServiceList = HashSet<Service>;
type EventServiceMap = HashMap<String, UniqueServiceList>;

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
                        payload: event.payload.clone()
                    };
                    client.post(service.endpoint.clone()).json::<VentrixEvent>(&body);
                }
            }
        }
    }
}
