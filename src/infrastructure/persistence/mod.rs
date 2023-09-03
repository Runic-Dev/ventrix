pub mod inmemory;
pub mod postgres;

use async_trait::async_trait;
use chrono::{Utc, DateTime};
use uuid::Uuid;
use std::{error::Error, fmt::Debug};

use crate::{
    common::types::{NewEventTypeRequest, VentrixEvent, PayloadSchema, ListenToEventReq, EventFulfillmentDetails},
    domain::models::service::{RegisterServiceRequest, Service},
};

#[async_trait]
pub trait Database: Debug + Send + Sync {
    async fn register_service(
        &self,
        service: &RegisterServiceRequest,
    ) -> Result<InsertDataResponse, Box<dyn Error>>;
    async fn remove_service(
        &self,
        service_name: &str,
    ) -> Result<DeleteDataResponse, Box<dyn Error>>;
    async fn register_event_type(
        &self,
        event_type: &NewEventTypeRequest,
    ) -> Result<InsertDataResponse, Box<dyn Error>>;
    async fn get_service(&self, service_name: &str) -> Result<Service, Box<dyn Error>>;
    async fn save_published_event(
        &self,
        event: &VentrixEvent,
    ) -> Result<InsertDataResponse, Box<dyn Error>>;
    async fn fulfil_event(
        &self,
        event: &VentrixEvent,
    ) -> Result<UpdateDataResponse, Box<dyn Error>>;
    async fn register_service_for_event_type(
        &self,
        listen_to_event_req: &ListenToEventReq
    ) -> Result<InsertDataResponse, Box<dyn Error>>;
    async fn get_service_by_event_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<EventFulfillmentDetails>, Box<dyn Error + Sync + Send>>;
    async fn get_schema_for_event_type(
        &self,
        event_type: &str,
    ) -> Result<PayloadSchema, Box<dyn Error>>;
    async fn add_failed_event(
        &self,
        event: &VentrixEvent
    ) -> Result<InsertDataResponse, Box<dyn Error>>;
    async fn resolve_failed_event(
        &self,
        event_id: Uuid
    ) -> Result<UpdateDataResponse, Box<dyn Error>>;
    async fn update_retry_time(
        &self,
        event_id: Uuid,
        new_retry_time: DateTime<Utc>,
        retries: i16
    ) -> Result<UpdateDataResponse, Box<dyn Error>>;
    async fn get_failed_events(&self) -> 
        Result<Vec<VentrixEvent>, Box<dyn Error + Sync + Send>>;
}

pub enum InsertDataResponse {
    InMemory,
    Postgres(u64),
}

pub enum DeleteDataResponse {
    InMemory,
    Postgres(u64),
}

pub enum UpdateDataResponse {
    InMemory,
    Postgres(u64),
}
