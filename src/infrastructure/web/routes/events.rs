use crate::application::queue_service::ventrix_queue::VentrixQueue;
use crate::common::types::{FeatureFlagConfig, VentrixEvent};
use crate::infrastructure::persistence::Database;
use crate::common::schema_validator::is_valid_property_def;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[tracing::instrument(
    name = "Registering new event type",
    fields (
        %event_type_to_register.name,
        %event_type_to_register.description,
        %event_type_to_register.payload_definition
    )
)]
pub async fn register_new_event_type(
    mut event_type_to_register: web::Json<NewEventTypeRequest>,
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
    match database.get_ref().get_service(&listen_request.service_name).await {
        Ok(service) => {
            match queue
                .listen_to_event(&service.name, &listen_request.event_type)
                .await
            {
                Ok(_) => {
                    HttpResponse::Created().json(ListenToEventResponse {
                        message: format!(
                            "Service {} successfully registered to listen to event type {}",
                            service,
                            listen_request.event_type.clone()
                        ),
                    })
                }
                Err(_) => HttpResponse::Ok()
                    .json(ListenToEventResponse {
                        message: format!(
                            "Service {} was already registered to listen to event type {}",
                            service,
                            listen_request.event_type.clone()
                        ),
                    }),
            }
        }
        Err(err) => {
            let response = json!({
                "message": "There was a problem finding the service on the database",
                "error": format!("{}", err)
            });
            HttpResponse::InternalServerError().json(response)
        }
    }
}

#[derive(Serialize)]
pub struct ListenToEventResponse {
    message: String,
}

#[tracing::instrument(name = "Publishing event")]
pub async fn publish_event(
    publish_event_req: web::Json<PublishEventRequest>,
    queue: web::Data<VentrixQueue>,
    database: web::Data<Box<dyn Database>>
) -> HttpResponse {
    let queue = queue.get_ref();

    let event = VentrixEvent {
        id: Uuid::new_v4(),
        event_type: publish_event_req.event_type.clone(),
        payload: publish_event_req.payload.clone()
    };

    match queue.sender.send(event.clone()).await {
        Ok(_) => {
            match database.save_published_event(&event).await {
                Ok(_) => {
                    let response = json!({
                        "message": "Successfully added event to queue"
                    })
                        .to_string();
                    HttpResponse::Created().json(response)
                },
                Err(err) => {
                    HttpResponse::InternalServerError().json(json!({
                        "message": "Event added to queue but failed to be saved to the database",
                        "error": err.to_string()
                    }))
                }
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .reason("Unable to publish event")
            .finish(),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PublishEventRequest {
    pub event_type: String,
    pub payload: String
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
pub struct ListenToEvent {
    pub service_name: String,
    pub event_type: String,
}
