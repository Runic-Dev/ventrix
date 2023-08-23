use actix_web::{web, HttpResponse};

use crate::{
    application::queue_service::ventrix_queue::VentrixQueue, common::types::VentrixEvent,
    infrastructure::persistence::Database,
};

#[tracing::instrument(
    name = "New event added to queue",
    fields (
        %event.event_type,
        %event.payload
    )
)]
pub async fn enqueue_event(
    event: web::Json<VentrixEvent>,
    ventrix_queue: web::Data<VentrixQueue>,
    database: web::Data<dyn Database>,
) -> HttpResponse {
    tracing::info!("Adding event to the queue: {:?}", event);
    // TODO: Keep this logic encapsulated within the Ventrix queue
    match ventrix_queue
        .get_ref()
        .sender
        .send(event.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError()
            .reason("Unable to add event to queue")
            .finish(),
    }
}
