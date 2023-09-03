use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use uuid::Uuid;

pub fn datetime_utc_to_string<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date.to_rfc3339();
    serializer.serialize_str(&s)
}

pub fn string_to_datetime_utc<'de, D>(deserialize: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserialize)?;
    Ok(DateTime::parse_from_rfc3339(&s)
        .unwrap()
        .with_timezone(&Utc))
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RetryDetails {
    pub retry_count: i16,
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
    pub retry_details: Option<RetryDetails>,
}

impl VentrixEvent {
    pub fn from_failed_event(failed_event: &FailedEventRow) -> Self {
        Self {
            id: failed_event.id,
            event_type: failed_event.event_type.clone(),
            payload: failed_event.payload.clone(),
            retry_details: Some(RetryDetails {
                retry_count: failed_event.retries,
                retry_time: failed_event.retry_time,
            }),
        }
    }
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
    pub retries: usize,
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

#[derive(Debug, sqlx::FromRow)]
pub struct FailedEventRow {
    pub id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub retry_time: DateTime<Utc>,
    pub retries: i16,
}
