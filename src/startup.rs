use std::{collections::HashMap, net::TcpListener};

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing_actix_web::TracingLogger;

use crate::{
    database::Database,
    queue::VentrixQueue,
    routes::{events, health_check, services}, types::VentrixEvent,
};

pub async fn run(
    listener: TcpListener,
    database: Box<dyn Database>,
    _feature_flags: HashMap<&str, bool>,
) -> Result<Server, std::io::Error> {
    let database = web::Data::new(database);

    let (sender, receiver): (Sender<VentrixEvent>, Receiver<VentrixEvent>) =
        tokio::sync::mpsc::channel::<VentrixEvent>(50);
    let ventrix_queue = web::Data::new(VentrixQueue::new(sender).await);

    tokio::spawn(event_processor(receiver));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/service")
                            .route("/register", web::post().to(services::register_service))
                            .route("/remove", web::post().to(services::remove_service)),
                    )
                    .service(
                        web::scope("/events")
                            .route("/register", web::post().to(events::register_new_event_type))
                            .route("/publish", web::post().to(events::publish_event)),
                    ),
            )
            .app_data(Data::clone(&database))
            .app_data(Data::clone(&ventrix_queue))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

async fn event_processor(mut receiver: Receiver<VentrixEvent>) {
    while let Some(event) = receiver.recv().await {
        tracing::info!("Processing event: {}", event.event_type);
    }
}
