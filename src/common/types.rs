use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VentrixEvent {
    pub id: Uuid,
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

pub type FeatureFlagConfig = HashMap<String, bool>;

#[derive(Serialize)]
pub struct ListenToEventResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PublishEventRequest {
    pub event_type: String,
    pub payload: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewEventTypeRequest {
    pub name: String,
    pub description: String,
    pub payload_definition: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventType {
    pub id: String,
    pub name: String,
    pub description: String,
    pub payload_definition: Value,
}

#[derive(Debug, Deserialize)]
pub struct ListenToEvent {
    pub service_name: String,
    pub event_type: String,
}
