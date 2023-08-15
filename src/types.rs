use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VentrixEvent {
    pub event_type: String,
    pub payload: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ServiceDetails {
    endpoint: String,
}

impl ServiceDetails {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeDetails {
    description: String,
    payload_def: String,
}

impl EventTypeDetails {
    pub fn new(description: String, payload_def: String) -> Self {
        Self {
            description,
            payload_def,
        }
    }
}
