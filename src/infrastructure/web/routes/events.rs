use crate::application::queue_service::ventrix_queue::VentrixQueue;
use crate::common::errors::InvalidPropertyDef;
use crate::common::schema_validator::is_valid_property_def;
use crate::common::types::{
    FeatureFlagConfig, ListenToEventReq, ListenToEventResponse, NewEventTypeRequest,
    PublishEventRequest, VentrixEvent,
};
use crate::infrastructure::persistence::Database;
use actix_web::{web, HttpResponse};
use serde_json::{json, Value};
use uuid::Uuid;
use valico::json_schema::Scope;

fn set_payload(
    feature_flags: web::Data<FeatureFlagConfig>,
    mut payload_def: Value,
) -> Result<Value, InvalidPropertyDef> {
    if feature_flags
        .get("validate_event_def")
        .is_some_and(|feature_on| *feature_on)
    {
        return is_valid_property_def(&mut payload_def);
    };

    Ok(payload_def)
}

#[tracing::instrument(
    name = "Registering new event type",
    fields (
        %event_type_to_register.name,
        %event_type_to_register.description,
        %event_type_to_register.payload_definition
    )
)]
pub async fn register_new_event_type(
    event_type_to_register: web::Json<NewEventTypeRequest>,
    database: web::Data<dyn Database>,
    feature_flags: web::Data<FeatureFlagConfig>,
) -> HttpResponse {
    let database = database.get_ref();

    match set_payload(
        feature_flags,
        event_type_to_register.payload_definition.clone(),
    ) {
        Ok(_) => {
            let database_response = database.register_event_type(&event_type_to_register).await;

            match database_response {
                Ok(_) => {
                    let json_response = json!(
                        {
                            "name": event_type_to_register.name,
                            "description" : event_type_to_register.description,
                            "payload_description": event_type_to_register.payload_definition
                        }
                    );
                    HttpResponse::Created().json(json_response)
                }
                Err(err) => HttpResponse::BadRequest().json(err.to_string()),
            }
        }
        Err(err) => {
            let response = json!({
                "message": "Issue validating payload",
                "error": err.to_string()
            });
            HttpResponse::BadRequest().json(response)
        }
    }
}

#[tracing::instrument(name = "Listening to event type", fields())]
pub async fn listen_to_event(
    listen_request: web::Json<ListenToEventReq>,
    database: web::Data<dyn Database>,
) -> HttpResponse {
    match database
        .get_ref()
        .register_service_for_event_type(&listen_request)
        .await
    {
        Ok(_) => HttpResponse::Created().json(ListenToEventResponse {
            message: format!(
                "Service {} successfully registered to listen to event type {}",
                &listen_request.service_name, &listen_request.event_type
            ),
        }),
        Err(err) => {
            let err_string = err.to_string();
            match err.downcast::<sqlx::Error>().is_ok_and(|sqlx_err| {
                if let sqlx::Error::RowNotFound = *sqlx_err {
                    return true;
                }
                false
            }) {
                true => {
                    let response = json!({
                        "message": "RowNotFound error returned from server. Is the event_type in your request spelt correctly?"
                    });
                    HttpResponse::BadRequest().json(response)
                }
                false => HttpResponse::InternalServerError().json(err_string),
            }
        }
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
        retry_details: None,
    };

    if let Err(err) = database.save_published_event(&event).await {
        let response = json!({
            "message": err.to_string()
        });
        return HttpResponse::InternalServerError().json(response);
    };

    let schema = match database
        .get_schema_for_event_type(&publish_event_req.event_type)
        .await
    {
        Ok(schema_obj) => schema_obj.payload_definition,
        Err(err) => {
            let response = json!({
                "message": "Unable to get schema string for event type",
                "err": err.to_string()
            });
            return HttpResponse::BadRequest().json(response);
        }
    };

    let schema_value: Value = match serde_json::from_str(&schema) {
        Ok(schema_as_value) => schema_as_value,
        Err(err) => {
            let response = json!({
                "message": "Couldn't parse Value from schema string",
                "err": err.to_string(),
            });
            return HttpResponse::BadRequest().json(response);
        }
    };

    let mut scope = Scope::new();
    let scoped_schema = match scope.compile_and_return(schema_value.clone(), false) {
        Ok(scoped_schema) => scoped_schema,
        Err(err) => {
            let response = json!({
                "message": "Couldn't compile scoped schema from schema value",
                "err": err.to_string()
            });
            return HttpResponse::BadRequest().json(response);
        }
    };

    let payload_value: Value = match serde_json::from_str(&event.payload) {
        Ok(payload_as_value) => payload_as_value,
        Err(err) => {
            let response = json!({
                "message": "Couldn't parse Value from payload string",
                "err": err.to_string()
            });
            return HttpResponse::BadRequest().json(response);
        }
    };

    let validation = scoped_schema.validate(&payload_value);

    if !validation.is_strictly_valid() {
        let response = json!({
            "message": "Payload did not match the event type payload definition",
            "expected_payload_schema": schema_value
        });
        HttpResponse::BadRequest().json(response)
    } else {
        match queue.publish_event(event).await {
            Ok(_) => HttpResponse::Created().finish(),
            Err(err) => {
                let response = json!({
                    "message": err.to_string()
                });
                HttpResponse::InternalServerError().json(response)
            }
        }
    }
}
