use chrono::DateTime;
use std::collections::HashMap;
use std::error::Error;

use crate::common::errors::EventNotFoundError;
use crate::common::errors::EventTypeAlreadyExistsError;
use crate::common::errors::ServiceAlreadyExistsError;
use crate::common::errors::ServiceNotFoundError;
use crate::common::types::EventFulfillmentDetails;
use crate::common::types::ListenToEventReq;
use crate::common::types::NewEventTypeRequest;
use crate::common::types::PayloadSchema;
use crate::common::types::{EventTypeDetails, VentrixEvent};
use crate::domain::models::service::RegisterServiceRequest;
use crate::domain::models::service::Service;
use async_trait::async_trait;
use chrono::Utc;
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
                    url: reg_service_req.url.clone(),
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
        listen_to_event_req: &ListenToEventReq,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let mut service_to_event_type_lock = self.event_type_to_service.lock().await;
        let service_register_lock = self.service_register.lock().await;
        let service = service_register_lock
            .get(&listen_to_event_req.service_name.clone())
            .ok_or_else(|| ServiceNotFoundError::new(&listen_to_event_req.service_name.clone()))?;
        service_to_event_type_lock
            .get_mut(&listen_to_event_req.event_type.clone())
            .ok_or_else(|| ServiceNotFoundError::new(&listen_to_event_req.service_name))
            .map(|service_vec| service_vec.push(service.clone()))?;
        Ok(InsertDataResponse::InMemory)
    }

    async fn get_service_by_event_type(
        &self,
        _event_type: &str,
    ) -> Result<Vec<EventFulfillmentDetails>, Box<dyn Error + Sync + Send>> {
        todo!()
    }

    async fn get_schema_for_event_type(
        &self,
        _event_type: &str,
    ) -> Result<PayloadSchema, Box<dyn Error>> {
        todo!()
    }

    async fn resolve_failed_event(
        &self,
        _event_id: Uuid,
    ) -> Result<UpdateDataResponse, Box<dyn Error>> {
        todo!()
    }

    async fn add_failed_event(
        &self,
        _event: &VentrixEvent,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        todo!()
    }

    async fn update_retry_time(
        &self,
        _event_id: Uuid,
        _new_retry_time: DateTime<Utc>,
        _retries: i16,
    ) -> Result<UpdateDataResponse, Box<dyn Error>> {
        todo!()
    }
    async fn get_failed_events(&self) -> Result<Vec<VentrixEvent>, Box<dyn Error + Sync + Send>> {
        todo!()
    }
}
