use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VentrixEvent {
    pub event_type: String,
    pub payload: String,
}

pub struct ServiceDetails {
    endpoint: String
}

pub struct EventTypeDetails {
    description: String,
    payload_def: String
}

