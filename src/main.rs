pub mod common;

use common::configuration::get_configuration;
use common::telemetry::{get_subscriber, init_tracing_subscriber};
use common::types::FeatureFlagConfig;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::time::sleep;
use std::collections::HashMap;
use std::net::TcpListener;
use std::time::Duration;
use ventrix::infrastructure::persistence::inmemory::InMemoryDatabase;
use ventrix::infrastructure::persistence::postgres::PostgresDatabase;
use ventrix::infrastructure::persistence::Database;
use ventrix::infrastructure::web::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("ventrix".into(), "info".into(), std::io::stdout);
    init_tracing_subscriber(subscriber);

    let feature_flags: FeatureFlagConfig = HashMap::from([
        (String::from("persistence"), true),
        (String::from("validate_event_def"), false),
    ]);

    let configuration = get_configuration().expect("Failed to read configuration");

    let database: Box<dyn Database> = match feature_flags.get("persistence") {
        Some(persistence_true) => {
            if *persistence_true {
                let pool = PgPool::connect_lazy(
                    configuration.database.connection_string().expose_secret(),
                )
                .expect("Failed to connect to Postgres");

                if let Err(err) = wait_for_db(configuration.database.connection_string().expose_secret()).await {
                    panic!("Received error: {}", err)
                }

                // TODO: Move migrations to run before container starts up, iSEVEN says so
                if let Err(err) = sqlx::migrate!("./migrations").run(&pool).await {
                    panic!("{}", err.to_string())
                }
                Box::new(PostgresDatabase::new(pool))
            } else {
                Box::<InMemoryDatabase>::default()
            }
        }
        None => Box::<InMemoryDatabase>::default()
    };

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, database, feature_flags).await?.await
}

async fn wait_for_db(connection_string: &str) -> Result<(), sqlx::Error> {
    let mut retries = 5;

    while retries > 0 {
        if PgPoolOptions::new().connect(connection_string).await.is_ok()
        {
            return Ok(());
        }

        retries -= 1;
        sleep(Duration::from_secs(5)).await;
    }

    Err(sqlx::Error::Configuration("Database unavailable".into()))
}
