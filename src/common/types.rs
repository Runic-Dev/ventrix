use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct VentrixEvent {
    pub event_type: String,
    pub payload: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ServiceDetails {
    pub endpoint: String,
}

impl ServiceDetails {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeDetails {
    description: String,
    payload_def: Value,
}

impl EventTypeDetails {
    pub fn new(description: String, payload_def: Value) -> Self {
        Self {
            description,
            payload_def,
        }
    }
}
