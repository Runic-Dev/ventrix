use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

use crate::models::service::{Service, RegisterServiceRequest};

#[tracing::instrument(
    name = "Registering a new service",
    fields(
        name = %service_to_register.name,
        url = %service_to_register.endpoint
    )
)]
pub async fn register_service(
    service_to_register: web::Json<RegisterServiceRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let uuid = Uuid::new_v4();
    let query_span = tracing::info_span!("Saving new service");
    match sqlx::query!(
        r#"
        INSERT INTO services (id, name, url)
        VALUES ($1, $2, $3)
        "#,
        uuid,
        service_to_register.name,
        service_to_register.endpoint
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
