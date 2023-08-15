use crate::{
    errors::{EventTypeAlreadyExistsError, ServiceAlreadyExistsError},
    models::service::RegisterServiceRequest,
    routes::events::NewEventType,
    types::{EventTypeDetails, ServiceDetails},
};
use actix_web::web;
use async_trait::async_trait;
use std::fmt::Debug;
use std::{collections::HashMap, error::Error};
use uuid::Uuid;

use sqlx::PgPool;
use tokio::sync::Mutex;

#[async_trait]
pub trait Database: Debug + Send + Sync {
    async fn register_service(
        &self,
        service_to_register: web::Json<RegisterServiceRequest>,
    ) -> Result<(String, ServiceDetails), Box<dyn Error>>;
    async fn register_event_type(
        &self,
        event_type: NewEventType,
    ) -> Result<(String, EventTypeDetails), Box<dyn Error>>;
}

#[derive(Debug)]
pub struct PostgresDatabase {
    pub pool: PgPool,
}

impl PostgresDatabase {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Default)]
pub struct InMemoryDatabase {
    service_register: Mutex<HashMap<String, ServiceDetails>>,
    event_types: Mutex<HashMap<String, EventTypeDetails>>,
    event_type_to_services: HashMap<String, String>,
}

#[async_trait]
impl Database for InMemoryDatabase {
    async fn register_service(
        &self,
        service: web::Json<RegisterServiceRequest>,
    ) -> Result<(String, ServiceDetails), Box<dyn Error>> {
        let service = service.into_inner();
        let mut locked_service_register = self.service_register.lock().await;
        match locked_service_register.get::<String>(&service.name) {
            Some(_) => return Err(Box::new(ServiceAlreadyExistsError::new(service.name))),
            None => {
                let service_details = ServiceDetails::new(service.endpoint);
                let insert_result = locked_service_register.insert(service.name.clone(), service_details.clone());
                match insert_result {
                    Some(_) => return Err(Box::new(ServiceAlreadyExistsError::new(service.name))),
                    None => return Ok((service.name, service_details)),
                }
            },
        }
    }
    async fn register_event_type(
        &self,
        event_type: NewEventType,
    ) -> Result<(String, EventTypeDetails), Box<dyn Error>> {
        let event_type_details =
            EventTypeDetails::new(event_type.description, event_type.payload_description);
        let mut event_types_lock = self.event_types.lock().await;
        match event_types_lock.insert(event_type.name.clone(), event_type_details.clone()) {
            Some(_) => Err(Box::new(EventTypeAlreadyExistsError::new(event_type.name))),
            None => Ok((event_type.name, event_type_details.clone())),
        }
    }
}

#[async_trait]
impl Database for PostgresDatabase {
    async fn register_service(
        &self,
        service: web::Json<RegisterServiceRequest>,
    ) -> Result<(String, ServiceDetails), Box<dyn Error>> {
        let service = service.into_inner();
        let uuid = Uuid::new_v4();
        match sqlx::query!(
            r#"
        INSERT INTO services (id, name, url)
        VALUES ($1, $2, $3)
        "#,
            uuid,
            service.name.clone(),
            service.endpoint.clone()
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok((
                    service.name,
                    ServiceDetails::new(service.endpoint)
                )),
            Err(err) => {
                tracing::error!("Failed to execute query: {:?}", err);
                Err(Box::new(err))
            }
        }
    }

    async fn register_event_type(
        &self,
        event_type: NewEventType,
    ) -> Result<(String, EventTypeDetails), Box<dyn Error>> {
        let uuid = Uuid::new_v4();
        match sqlx::query!(
            r#"
        INSERT INTO event_types (id, name, description, payload_desc)
        VALUES ($1, $2, $3, $4)
        "#,
            uuid,
            event_type.name,
            event_type.description,
            event_type.payload_description
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok((
                event_type.name,
                EventTypeDetails::new(event_type.description, event_type.payload_description),
            )),
            Err(err) => {
                tracing::error!("Failed to execute query: {:?}", err);
                Err(Box::new(err))
            }
        }
    }
}
