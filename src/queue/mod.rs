use std::{collections::VecDeque};

use crate::{types::VentrixEvent, models::service::Service};

#[derive(Debug)]
pub struct VentrixQueue {
    pub queue: VecDeque<VentrixEvent>,
    services: Vec<Service>
}

