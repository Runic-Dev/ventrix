use crate::queue::VentrixQueue;
use crate::types::VentrixEvent;
use actix_web::{web, HttpResponse};

#[tracing::instrument(
    name = "New event added to queue",
    fields (
        %event_to_queue.event_type,
        %event_to_queue.payload
    )
)]
pub async fn enqueue_event(
    event_to_queue: web::Json<VentrixEvent>,
    ventrix_queue: web::Data<VentrixQueue>,
) -> HttpResponse {
    tracing::info!("Adding event to the queue: {:?}", event_to_queue);
    let mut locked_queue = ventrix_queue.queue.lock().await;

    locked_queue.push_back(event_to_queue.into_inner());
    HttpResponse::Created().finish()
}
