use crate::{models::service::Service, types::VentrixEvent};
use std::collections::VecDeque;

use tokio::sync::Mutex;


#[derive(Debug, Default)]
pub struct VentrixQueue {
    pub queue: Mutex<VecDeque<VentrixEvent>>,
    services: Vec<Service>
}

