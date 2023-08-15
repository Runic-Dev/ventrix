use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    database::Database,
    queue::ListenToEventResult::{Existed, NewEntry},
    queue::VentrixQueue,
    types::VentrixEvent,
};

#[tracing::instrument(
    name = "Registering new event type",
    fields (
        %event_type_to_register.name,
        %event_type_to_register.description,
        %event_type_to_register.payload_description
    )
)]
pub async fn register_new_event_type(
    event_type_to_register: web::Json<NewEventType>,
    database: web::Data<Box<dyn Database>>,
) -> HttpResponse {
    let database = database.get_ref();
    let event_type_to_register = event_type_to_register.into_inner();
    let database_response = database.register_event_type(&event_type_to_register).await;

    match database_response {
        Ok(_) => {
            let json_response = json!(
                {
                    "name": event_type_to_register.name,
                    "description" : event_type_to_register.description,
                    "payload_description": event_type_to_register.payload_description
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
                NewEntry => HttpResponse::Created().json(ListenToEventResponse {
                    message: format!(
                        "Service {} successfully registered to listen to event type {}",
                        service,
                        listen_request.event_type.clone()
                    ),
                }),
                Existed => HttpResponse::Ok().json(ListenToEventResponse {
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
pub struct NewEventType {
    pub name: String,
    pub description: String,
    pub payload_description: String,
}

#[derive(Debug, Deserialize)]
pub struct ListenToEvent {
    pub service_name: String,
    pub event_type: String,
}
