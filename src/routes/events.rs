use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;

use crate::{database::Database, queue::VentrixQueue, types::VentrixEvent};

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
    let database_response = database
        .register_event_type(event_type_to_register.into_inner())
        .await;

    match database_response {
        Ok((name, details)) => {
            let json_response = json!({
                "name": name,
                "event_type_details": details
            })
            .to_string();
            HttpResponse::Created().json(json_response)
        }
        Err(err) => HttpResponse::BadRequest().json(err.to_string()),
    }
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
        Err(_) => HttpResponse::InternalServerError().reason("Unable to publish event").finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct NewEventType {
    pub name: String,
    pub description: String,
    pub payload_description: String,
}
