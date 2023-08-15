use crate::types::{EventTypeDetails, ServiceDetails};
use std::collections::HashMap;

use sqlx::PgPool;

pub enum DatabaseOption {
    Postgres(PostgresDatabase),
    InMemory(InMemoryDatabase)
}

pub struct PostgresDatabase {
    pool: PgPool
}

impl PostgresDatabase {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

#[derive(Default)]
pub struct InMemoryDatabase {
    service_register: HashMap<String, ServiceDetails>,
    event_types: HashMap<String, EventTypeDetails>,
    event_type_to_services: HashMap<String, String>
}
