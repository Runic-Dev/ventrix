use std::net::TcpListener;

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use tracing_actix_web::TracingLogger;

use crate::{
    application::queue_service::ventrix_queue::VentrixQueue, common::types::FeatureFlagConfig,
    infrastructure::persistence::Database,
};

use super::routes::{events, health_check, services};

pub async fn run(
    listener: TcpListener,
    database: web::Data<dyn Database>,
    feature_flags: FeatureFlagConfig,
) -> Result<Server, std::io::Error> {
    let ventrix_queue = VentrixQueue::new(database.clone()).await;
    let ventrix_queue = web::Data::new(ventrix_queue);
    let feature_flags = web::Data::new(feature_flags);

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
                            .route("/publish", web::post().to(events::publish_event))
                            .route("/listen", web::post().to(events::listen_to_event)),
                    ),
            )
            .app_data(database.clone())
            .app_data(Data::clone(&ventrix_queue))
            .app_data(Data::clone(&feature_flags))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
