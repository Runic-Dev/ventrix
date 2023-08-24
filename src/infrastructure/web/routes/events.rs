use crate::application::queue_service::ventrix_queue::VentrixQueue;
use crate::common::schema_validator::is_valid_property_def;
use crate::common::types::{
    FeatureFlagConfig, ListenToEvent, ListenToEventResponse, NewEventTypeRequest,
    PublishEventRequest, VentrixEvent,
};
use crate::infrastructure::persistence::Database;
use actix_web::{web, HttpResponse};
use serde_json::json;
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
    database: web::Data<dyn Database>,
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
    database: web::Data<dyn Database>,
) -> HttpResponse {
    match database
        .get_ref()
        .register_service_for_event_type(&listen_request.service_name, &listen_request.event_type)
        .await
    {
        Ok(_) => HttpResponse::Created().json(ListenToEventResponse {
            message: format!(
                "Service {} successfully registered to listen to event type {}",
                &listen_request.service_name, &listen_request.event_type
            ),
        }),
        Err(err) => HttpResponse::Ok().json(ListenToEventResponse {
            message: format!(
                "Failed subscribing service {} to event type {}. Error from database: {}",
                &listen_request.service_name, &listen_request.event_type, err
            ),
        }),
    }
}

#[tracing::instrument(name = "Publishing event")]
pub async fn publish_event(
    publish_event_req: web::Json<PublishEventRequest>,
    queue: web::Data<VentrixQueue>,
    database: web::Data<dyn Database>,
) -> HttpResponse {
    let queue = queue.get_ref();

    let event = VentrixEvent {
        id: Uuid::new_v4(),
        event_type: publish_event_req.event_type.clone(),
        payload: publish_event_req.payload.clone(),
    };

    match queue.sender.send(event.clone()).await {
        Ok(_) => match database.save_published_event(&event).await {
            Ok(_) => {
                let response = json!({
                    "message": "Successfully added event to queue"
                })
                .to_string();
                HttpResponse::Created().json(response)
            }
            Err(err) => HttpResponse::InternalServerError().json(json!({
                "message": "Event added to queue but failed to be saved to the database",
                "error": err.to_string()
            })),
        },
        Err(_) => HttpResponse::InternalServerError()
            .reason("Unable to publish event")
            .finish(),
    }
}
