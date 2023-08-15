use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use ventrix::configuration::get_configuration;
use ventrix::startup::run;
use ventrix::telemetry::{get_subscriber, init_tracing_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("ventrix".into(), "info".into(), std::io::stdout);
    init_tracing_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_pool =
        PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
            .expect("Failed to connect to Postgres");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
