use crate::{models::service::Service, types::VentrixEvent};
use std::collections::{HashMap, HashSet};

use tokio::sync::{
    mpsc::Sender,
    Mutex,
};

#[derive(Debug)]
pub struct VentrixQueue {
    pub sender: Sender<VentrixEvent>,
    pub event_type_to_services: Mutex<HashMap<String, HashSet<Service>>>,
}

impl VentrixQueue {
    pub async fn new(sender: Sender<VentrixEvent>) -> Self {
        Self {
            sender,
            event_type_to_services: Mutex::new(HashMap::<String, HashSet<Service>>::default()),
        }
    }
}
