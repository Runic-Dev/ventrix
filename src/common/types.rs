use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde_json::Value;
use uuid::Uuid;

type RetryDetailsValue = Value;

pub fn datetime_utc_to_string<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    let s = date.to_rfc3339();
    serializer.serialize_str(&s)
}

pub fn string_to_datetime_utc<'de, D>(deserialize: D) -> Result<DateTime<Utc>, D::Error> where D: Deserializer<'de> {
    let s = String::deserialize(deserialize)?;
    Ok(DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc))
}

#[derive(Deserialize, Serialize)]
pub struct RetryDetails {
    pub retry_count: usize,
    #[serde(
        serialize_with = "datetime_utc_to_string",
        deserialize_with = "string_to_datetime_utc"
    )]
    pub retry_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VentrixEvent {
    pub id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub retry_details: Option<RetryDetailsValue>,
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

impl Display for PublishEventRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ event_type: {}, payload: {} }}",
            self.event_type, self.payload
        )
    }
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
pub struct ListenToEventReq {
    pub service_name: String,
    pub event_type: String,
    pub endpoint: String,
}

#[derive(sqlx::FromRow)]
pub struct PayloadSchema {
    pub payload_definition: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EventFulfillmentDetails {
    pub name: String,
    pub url: String,
    pub endpoint: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct FailedEvent {
    pub id: Uuid,
    pub event_id: Uuid,
    pub details: Value,
    pub retry_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub retires: usize,
    pub resolved_at: DateTime<Utc>,
}

impl Display for EventFulfillmentDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ name: {}, url: {}, endpoint: {}}}",
            self.name, self.url, self.endpoint
        )
    }
}

pub type VentrixQueueResponseMessage = String;

#[derive(Debug)]
pub enum VentrixQueueResponse {
    PublishedAndSaved(VentrixQueueResponseMessage),
    PublishedNotSaved(VentrixQueueResponseMessage),
}
