use actix_web::{web, HttpResponse};
use serde_json::json;

use crate::{database::Database, models::service::Service, types::ServiceDetails};

use super::DeleteServiceRequest;

#[tracing::instrument(
    name = "Registering a new service",
    fields(
        name = %service.name,
        url = %service.endpoint
    )
)]
pub async fn register_service(
    service: web::Json<Service>,
    database: web::Data<Box<dyn Database>>,
) -> HttpResponse {
    let service = service.into_inner();
    match database.register_service(&service).await {
        Ok(_) => {
            let response = json!({
                "name": service.name,
                "service_details": ServiceDetails {
                    endpoint: service.endpoint
                }
            })
            .to_string();
            HttpResponse::Created().json(response)
        }
        Err(err) => HttpResponse::BadRequest().json(err.to_string()),
    }
}

#[tracing::instrument(
    name = "Removing a service",
    fields(
        name = %delete_service_req.name,
    )
)]
pub async fn remove_service(
    delete_service_req: web::Json<DeleteServiceRequest>,
    database: web::Data<Box<dyn Database>>,
) -> HttpResponse {
    match database.remove_service(&delete_service_req.name).await {
        Ok(_) => {
            let response = json!({
                "message": format!("Record successfully deleted for service: {}", delete_service_req.name)
            })
            .to_string();
            HttpResponse::NoContent().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
