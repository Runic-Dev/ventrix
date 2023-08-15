use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Eq, Hash, PartialEq, Clone, sqlx::FromRow, Deserialize)]
pub struct Service {
    pub name: String,
    pub endpoint: String,
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ name: {}, endpoint: {}}}", self.name, self.endpoint)
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterServiceRequest {
    pub name: String,
    pub endpoint: String,
}
