pub mod inmemory;
pub mod postgres;

use crate::{models::service::Service, routes::events::NewEventType};
use async_trait::async_trait;
use std::{error::Error, fmt::Debug};

#[async_trait]
pub trait Database: Debug + Send + Sync {
    async fn register_service(
        &self,
        service: &Service,
    ) -> Result<InsertDataResponse, Box<dyn Error>>;
    async fn remove_service(
        &self,
        service_name: &str
    ) -> Result<DeleteDataResponse, Box<dyn Error>>;
    async fn register_event_type(
        &self,
        event_type: &NewEventType,
    ) -> Result<InsertDataResponse, Box<dyn Error>>;
    async fn get_service(&self, service_name: &str) -> Result<Service, Box<dyn Error>>;
}

pub enum InsertDataResponse {
    InMemory,
    Postgres(u64),
}

pub enum DeleteDataResponse {
    InMemory,
    Postgres(u64),
}
