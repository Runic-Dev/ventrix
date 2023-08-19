use crate::application::queue_service::ventrix_queue::VentrixQueue;
use crate::common::types::{FeatureFlagConfig, VentrixEvent};
use crate::infrastructure::persistence::Database;
use crate::{application, common::schema_validator::is_valid_property_def};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[tracing::instrument(
    name = "Registering new event type",
    fields (
        %event_type_to_register.name,
        %event_type_to_register.description,
        %event_type_to_register.payload_definition
    )
)]
pub async fn register_new_event_type(
    mut event_type_to_register: web::Json<NewEventType>,
    database: web::Data<Box<dyn Database>>,
    feature_flags: web::Data<FeatureFlagConfig>,
) -> HttpResponse {
    let database = database.get_ref();

    if let Some(payload_validation) = feature_flags.get("validate_event_def") {
        if *payload_validation {
            if let Err(err) = is_valid_property_def(&mut event_type_to_register.payload_definition)
            {
                return HttpResponse::BadRequest().json(err.to_string());
            }
        }
    }

    let database_response = database.register_event_type(&event_type_to_register).await;

    match database_response {
        Ok(_) => {
            let json_response = json!(
                {
                    "name": event_type_to_register.name,
                    "description" : event_type_to_register.description,
                    "payload_description": event_type_to_register.payload_definition
                }
            )
            .to_string();
            HttpResponse::Created().json(json_response)
        }
        Err(err) => HttpResponse::BadRequest().json(err.to_string()),
    }
}

#[tracing::instrument(name = "Listening to event type", fields())]
pub async fn listen_to_event(
    listen_request: web::Json<ListenToEvent>,
    queue: web::Data<VentrixQueue>,
    database: web::Data<Box<dyn Database>>,
) -> HttpResponse {
    match database.get_service(&listen_request.service_name).await {
        Ok(service) => {
            match queue
                .listen_to_event(&service, &listen_request.event_type)
                .await
            {
                application::queue_service::ListenToEventResult::NewEntry => {
                    HttpResponse::Created().json(ListenToEventResponse {
                        message: format!(
                            "Service {} successfully registered to listen to event type {}",
                            service,
                            listen_request.event_type.clone()
                        ),
                    })
                }
                application::queue_service::ListenToEventResult::Existed => HttpResponse::Ok()
                    .json(ListenToEventResponse {
                        message: format!(
                            "Service {} was already registered to listen to event type {}",
                            service,
                            listen_request.event_type.clone()
                        ),
                    }),
            }
        }
        Err(_) => {
            // TODO: Handle these cases better
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize)]
pub struct ListenToEventResponse {
    message: String,
}

#[tracing::instrument(name = "Publishing event")]
pub async fn publish_event(
    event: web::Json<VentrixEvent>,
    queue: web::Data<VentrixQueue>,
) -> HttpResponse {
    let queue = queue.get_ref();

    match queue.sender.send(event.into_inner()).await {
        Ok(_) => {
            // TODO: Find out why the json body isn't being sent
            let response = json!({
                "message": "Successfully added event to queue"
            })
            .to_string();
            HttpResponse::Created().json(response)
        }
        Err(_) => HttpResponse::InternalServerError()
            .reason("Unable to publish event")
            .finish(),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewEventTypeRequest {
    pub name: String,
    pub description: String,
    pub payload_definition: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewEventType {
    pub name: String,
    pub description: String,
    pub payload_definition: Value,
}

#[derive(Debug, Deserialize)]
pub struct ListenToEvent {
    pub service_name: String,
    pub event_type: String,
}
