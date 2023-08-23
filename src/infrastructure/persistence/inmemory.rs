use std::collections::HashMap;
use std::error::Error;

use crate::common::errors::EventNotFoundError;
use crate::common::errors::EventTypeAlreadyExistsError;
use crate::common::errors::EventTypeNotFoundError;
use crate::common::errors::ServiceAlreadyExistsError;
use crate::common::errors::ServiceNotFoundError;
use crate::common::types::{EventTypeDetails, VentrixEvent};
use crate::domain::models::service::RegisterServiceRequest;
use crate::domain::models::service::Service;
use crate::infrastructure::web::routes::events::NewEventTypeRequest;
use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::Database;
use super::DeleteDataResponse;
use super::InsertDataResponse;
use super::UpdateDataResponse;

#[derive(Debug, Default)]
pub struct InMemoryDatabase {
    service_register: Mutex<HashMap<String, Service>>,
    event_types: Mutex<HashMap<String, EventTypeDetails>>,
    published_events: Mutex<HashMap<Uuid, (VentrixEvent, bool)>>,
    event_type_to_service: Mutex<HashMap<String, Vec<Service>>>,
}

#[async_trait]
impl Database for InMemoryDatabase {
    async fn register_service(
        &self,
        reg_service_req: &RegisterServiceRequest,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let mut locked_service_register = self.service_register.lock().await;
        match locked_service_register.get::<String>(&reg_service_req.name) {
            Some(_) => {
                return Err(Box::new(ServiceAlreadyExistsError::new(
                    reg_service_req.name.clone(),
                )))
            }
            None => {
                let service = Service {
                    id: Uuid::new_v4(),
                    name: reg_service_req.name.clone(),
                    endpoint: reg_service_req.endpoint.clone()
                };
                let insert_result =
                    locked_service_register.insert(service.name.clone(), service.clone());
                match insert_result {
                    Some(_) => {
                        return Err(Box::new(ServiceAlreadyExistsError::new(
                            service.name.clone(),
                        )))
                    }
                    None => return Ok(InsertDataResponse::InMemory),
                }
            }
        }
    }

    async fn remove_service(
        &self,
        service_name: &str,
    ) -> Result<DeleteDataResponse, Box<dyn Error>> {
        let mut service_register_lock = self.service_register.lock().await;
        match service_register_lock.remove(service_name) {
            Some(_) => Ok(DeleteDataResponse::InMemory),
            None => Err(Box::new(ServiceNotFoundError::new(service_name))),
        }
    }

    async fn register_event_type(
        &self,
        new_event_type_req: &NewEventTypeRequest,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let event_type_details = EventTypeDetails::new(
            new_event_type_req.description.clone(),
            new_event_type_req.payload_definition.clone(),
        );
        let mut event_types_lock = self.event_types.lock().await;
        match event_types_lock.insert(new_event_type_req.name.clone(), event_type_details.clone()) {
            Some(_) => Err(Box::new(EventTypeAlreadyExistsError::new(
                new_event_type_req.name.clone(),
            ))),
            None => Ok(InsertDataResponse::InMemory),
        }
    }

    async fn get_service(&self, name: &str) -> Result<Service, Box<dyn Error>> {
        let service_register_lock = self.service_register.lock().await;
        match service_register_lock.get(name) {
            Some(service) => Ok(service.clone()),
            None => Err(Box::new(ServiceNotFoundError::new(name))),
        }
    }

    async fn save_published_event(
        &self,
        event: &VentrixEvent,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let mut events_vec = self.published_events.lock().await;
        events_vec.insert(event.id, (event.clone(), false));
        Ok(InsertDataResponse::InMemory)
    }

    async fn fulfil_event(
        &self,
        event: &VentrixEvent,
    ) -> Result<UpdateDataResponse, Box<dyn Error>> {
        let mut events_map_lock = self.published_events.lock().await;
        events_map_lock
            .get_mut(&event.id)
            .ok_or_else(|| EventNotFoundError::new(&event.id.to_string()))
            .map(|(_, is_fulfilled)| {
                *is_fulfilled = true;
            })?;
        Ok(UpdateDataResponse::InMemory)
    }

    async fn register_service_for_event_type(
        &self,
        service_name: &str,
        event_type_name: &str,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let mut service_to_event_type_lock = self.event_type_to_service.lock().await;
        let service_register_lock = self.service_register.lock().await;
        let service = service_register_lock
            .get(service_name.clone())
            .ok_or_else(|| ServiceNotFoundError::new(service_name.clone()))?;
        service_to_event_type_lock
            .get_mut(event_type_name.clone())
            .ok_or_else(|| ServiceNotFoundError::new(service_name))
            .map(|service_vec| service_vec.push(service.clone()))?;
        Ok(InsertDataResponse::InMemory)
    }

    async fn get_service_by_event_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<Service>, Box<dyn Error + Send>> {
        let event_type_to_service_lock = self.event_type_to_service.lock().await;
        match event_type_to_service_lock
            .get(event_type)
            .ok_or_else(|| Box::new(EventTypeNotFoundError::new(event_type))) {
                Ok(service_vec) => Ok(service_vec.clone()),
                Err(err) => Err(err)
        }
    }
}
