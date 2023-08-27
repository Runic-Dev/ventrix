use std::fmt::Display;

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Eq, Hash, PartialEq, Clone, sqlx::FromRow, Deserialize)]
pub struct Service {
    pub id: Uuid,
    pub name: String,
    pub url: String,
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ name: {}, endpoint: {}}}", self.name, self.url)
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterServiceRequest {
    pub name: String,
    pub url: String,
}
