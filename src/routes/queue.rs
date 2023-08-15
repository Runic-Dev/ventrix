use std::borrow::BorrowMut;

use actix_web::{web, HttpResponse};

use crate::{types::VentrixEvent, queue::VentrixQueue};

#[tracing::instrument(
    name = "New event added to queue",
    fields (
        %event_to_queue.event_type,
        %event_to_queue.payload
    )
)]
pub async fn enqueue_event(event_to_queue: web::Json<VentrixEvent>, mut ventrix_queue: web::Data<VentrixQueue>) -> HttpResponse {
    tracing::info!("Adding event to the queue: {:?}", event_to_queue);
    let ventrix_q = ventrix_queue.borrow_mut();
    let mut locked_queue = ventrix_q.queue.lock().await;
    
    locked_queue.push_back(event_to_queue.into_inner());
    HttpResponse::Created().finish()
}
