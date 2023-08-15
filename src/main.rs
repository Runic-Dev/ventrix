use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::TcpListener;
use ventrix::database::{DatabaseOption, PostgresDatabase, InMemoryDatabase};
use ventrix::configuration::get_configuration;
use ventrix::startup::run;
use ventrix::telemetry::{get_subscriber, init_tracing_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("ventrix".into(), "info".into(), std::io::stdout);
    init_tracing_subscriber(subscriber);

    let feature_flags: HashMap<&str, bool> = HashMap::from([("persistence", false)]);

    let configuration = get_configuration().expect("Failed to read configuration");

    let database = match feature_flags.get("persistence") {
        Some(persistence_true) => {
            if *persistence_true {
            let pool =
                PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
                    .expect("Failed to connect to Postgres");
                DatabaseOption::Postgres(PostgresDatabase::new(pool))
            } else {
                DatabaseOption::InMemory(InMemoryDatabase::default())
            }
        }
        None => DatabaseOption::InMemory(InMemoryDatabase::default()),
    };

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, database, feature_flags)?.await
}
