use std::{collections::HashMap, error::Error};

use crate::{
    errors::{EventTypeAlreadyExistsError, ServiceAlreadyExistsError, ServiceNotFoundError},
    models::service::Service,
    routes::events::NewEventType,
    types::{EventTypeDetails, ServiceDetails},
};
use async_trait::async_trait;
use tokio::sync::Mutex;

use super::{Database, InsertDataResponse, DeleteDataResponse};

#[derive(Debug, Default)]
pub struct InMemoryDatabase {
    service_register: Mutex<HashMap<String, ServiceDetails>>,
    event_types: Mutex<HashMap<String, EventTypeDetails>>,
}

#[async_trait]
impl Database for InMemoryDatabase {
    async fn register_service(
        &self,
        service: &Service,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let mut locked_service_register = self.service_register.lock().await;
        match locked_service_register.get::<String>(&service.name) {
            Some(_) => return Err(Box::new(ServiceAlreadyExistsError::new(service.name.clone()))),
            None => {
                let service_details = ServiceDetails::new(service.endpoint.clone());
                let insert_result =
                    locked_service_register.insert(service.name.clone(), service_details.clone());
                match insert_result {
                    Some(_) => return Err(Box::new(ServiceAlreadyExistsError::new(service.name.clone()))),
                    None => return Ok(InsertDataResponse::InMemory),
                }
            }
        }
    }

    async fn register_event_type(
        &self,
        event_type: &NewEventType,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let event_type_details =
            EventTypeDetails::new(event_type.description.clone(), event_type.payload_description.clone());
        let mut event_types_lock = self.event_types.lock().await;
        match event_types_lock.insert(event_type.name.clone(), event_type_details.clone()) {
            Some(_) => Err(Box::new(EventTypeAlreadyExistsError::new(event_type.name.clone()))),
            None => Ok(InsertDataResponse::InMemory),
        }
    }

    async fn get_service(&self, name: &str) -> Result<Service, Box<dyn Error>> {
        let service_register_lock = self.service_register.lock().await;
        match service_register_lock.get(name) {
            Some(service_details) => Ok(Service {
                name: String::from(name),
                endpoint: service_details.endpoint.clone(),
            }),
            None => Err(Box::new(ServiceNotFoundError::new(name))),
        }
    }

    async fn remove_service(&self, service_name: &str) -> Result<DeleteDataResponse, Box<dyn Error>> {
        let mut service_register_lock = self.service_register.lock().await;
        match service_register_lock.remove(service_name) {
            Some(_) => Ok(DeleteDataResponse::InMemory),
            None => Err(Box::new(ServiceNotFoundError::new(service_name)))
        }
    }
}
