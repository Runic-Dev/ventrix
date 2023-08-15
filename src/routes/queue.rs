use actix_web::{web, HttpResponse};
use tokio::sync::Mutex;

use crate::{types::VentrixEvent, queue::VentrixQueue};

#[tracing::instrument(
    name = "New event added to queue",
    fields (
        %event_to_queue.event_type,
        %event_to_queue.payload
    )
)]
pub async fn enqueue_event(event_to_queue: VentrixEvent, ventrix_queue: web::Data<Mutex<VentrixQueue>>) -> HttpResponse {
    tracing::info!("Adding event to the queue: {:?}", event_to_queue);
    let mut ventrix_q_lock = ventrix_queue.lock().await;
    ventrix_q_lock.queue.push_back(event_to_queue);
    HttpResponse::Created().finish()
}
