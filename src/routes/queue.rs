use crate::queue::VentrixQueue;
use crate::types::VentrixEvent;
use actix_web::{web, HttpResponse};

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
) -> HttpResponse {
    tracing::info!("Adding event to the queue: {:?}", event);
    // TODO: Introduce some encapsulation, you bladdy heathen!
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
