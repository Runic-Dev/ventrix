pub mod inmemory;
pub mod postgres;

use async_trait::async_trait;
use std::{error::Error, fmt::Debug};

use crate::{
    common::types::{NewEventTypeRequest, VentrixEvent, PayloadSchema},
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
        service_name: &str,
        event_type_name: &str,
    ) -> Result<InsertDataResponse, Box<dyn Error>>;
    async fn get_service_by_event_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<Service>, Box<dyn Error + Send>>;
    async fn get_schema_for_event_type(
        &self,
        event_type: &str,
    ) -> Result<PayloadSchema, Box<dyn Error>>;
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
