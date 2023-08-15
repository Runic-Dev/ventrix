use std::collections::VecDeque;

use tokio::sync::Mutex;

use crate::{types::VentrixEvent, models::service::Service};

#[derive(Debug, Default)]
pub struct VentrixQueue {
    pub queue: Mutex<VecDeque<VentrixEvent>>,
    services: Vec<Service>
}

