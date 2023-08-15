pub mod common;

use common::configuration::get_configuration;
use common::telemetry::{get_subscriber, init_tracing_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::TcpListener;
use ventrix::infrastructure::persistence::inmemory::InMemoryDatabase;
use ventrix::infrastructure::persistence::postgres::PostgresDatabase;
use ventrix::infrastructure::persistence::Database;
use ventrix::infrastructure::web::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("ventrix".into(), "info".into(), std::io::stdout);
    init_tracing_subscriber(subscriber);

    let feature_flags: HashMap<&str, bool> = HashMap::from([("persistence", false)]);

    let configuration = get_configuration().expect("Failed to read configuration");

    let database: Box<dyn Database> = match feature_flags.get("persistence") {
        Some(persistence_true) => {
            if *persistence_true {
                let pool = PgPool::connect_lazy(
                    configuration.database.connection_string().expose_secret(),
                )
                .expect("Failed to connect to Postgres");
                Box::new(PostgresDatabase::new(pool))
            } else {
                Box::<InMemoryDatabase>::default()
            }
        }
        None => Box::<InMemoryDatabase>::default(),
    };

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, database, feature_flags).await?.await
}
