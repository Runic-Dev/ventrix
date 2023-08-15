use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

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
    pool: web::Data<PgPool>
) -> HttpResponse {

    let uuid = Uuid::new_v4();
    let query_span = tracing::info_span!("Saving new event type");
    match sqlx::query!(
        r#"
        INSERT INTO event_types (id, name, description, payload_desc)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid,
        event_type_to_register.name,
        event_type_to_register.description,
        event_type_to_register.payload_description
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

#[derive(Debug, Deserialize)]
pub struct NewEventType {
    name: String,
    description: String,
    payload_description: String
}


