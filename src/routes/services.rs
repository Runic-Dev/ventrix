use actix_web::{web, HttpResponse};
use serde_json::json;
use sqlx::PgPool;
use tracing::Instrument;

use crate::{models::service::{RegisterServiceRequest, Service}, database::Database};

#[tracing::instrument(
    name = "Registering a new service",
    fields(
        name = %service_to_register.name,
        url = %service_to_register.endpoint
    )
)]
pub async fn register_service(
    service_to_register: web::Json<RegisterServiceRequest>,
    database: web::Data<Box<dyn Database>>,
) -> HttpResponse {
    match database.register_service(service_to_register).await {
        Ok((name, details)) => {
            let response = json!({
                "name": name,
                "service_details": details
            }).to_string();
            HttpResponse::Created().json(response)
        },
        Err(err) => {
            HttpResponse::BadRequest().json(err.to_string())
        }
    }
}

#[tracing::instrument(
    name = "Removing a service",
    fields(
        id = %service_to_remove.id,
        name = %service_to_remove.name,
        url = %service_to_remove.endpoint
    )
)]
pub async fn remove_service(service_to_remove: Service, pool: web::Data<PgPool>) -> HttpResponse {
    let query_span = tracing::info_span!("Deleting service");
    match sqlx::query!(
        r#"
        DELETE from services WHERE id = $1
        "#,
        service_to_remove.id
    )
    .execute(pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => {
            tracing::error!("Failed to execute query: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
