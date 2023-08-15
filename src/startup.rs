use std::{net::TcpListener, sync::Mutex};

use actix_web::{dev::Server, web::{self, Data}, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{
    routes::{events, health_check, services, queue}, queue::VentrixQueue,
};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let ventrix_queue = web::Data::new(VentrixQueue::default());

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
                            .route("/publish", web::post().to(queue::enqueue_event))
                    ),
            )
            .app_data(db_pool.clone())
            .app_data(Data::clone(&ventrix_queue))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
