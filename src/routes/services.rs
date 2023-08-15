use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use tracing::Instrument;

use crate::models::service::Service;

#[tracing::instrument(
    name = "Registering a new service",
    fields(
        id = %service_to_register.id,
        name = %service_to_register.name,
        url = %service_to_register.url
    )
)]
pub async fn register_service(service_to_register: Service, pool: web::Data<PgPool>) -> HttpResponse {

    let query_span = tracing::info_span!("Saving new service");
    match sqlx::query!(
        r#"
        INSERT INTO services (id, name, url)
        VALUES ($1, $2, $3)
        "#,
        service_to_register.id,
        service_to_register.name,
        service_to_register.url
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
