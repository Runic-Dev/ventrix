use actix_web::{web, HttpResponse};
use serde_json::json;

use crate::{
    common::types::ServiceDetails, domain::models::service::RegisterServiceRequest,
    infrastructure::persistence::Database,
};

use super::DeleteServiceRequest;

#[tracing::instrument(
    name = "Registering a new service",
    fields(
        name = %reg_service_req.name,
        url = %reg_service_req.url
    )
)]
pub async fn register_service(
    reg_service_req: web::Json<RegisterServiceRequest>,
    database: web::Data<dyn Database>,
) -> HttpResponse {
    let reg_service_req = reg_service_req.into_inner();
    tracing::info!("Getting reference to database...");
    let database = database.get_ref();
    tracing::info!("Reference to database received!");
    match database.register_service(&reg_service_req).await {
        Ok(_) => {
            let response = json!({
                "name": reg_service_req.name,
                "service_details": ServiceDetails {
                    endpoint: reg_service_req.url
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
    database: web::Data<dyn Database>,
) -> HttpResponse {
    match database
        .get_ref()
        .remove_service(&delete_service_req.name)
        .await
    {
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
