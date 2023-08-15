use crate::domain::models::service::Service;
use crate::infrastructure::web::routes::events::NewEventType;
use std::error::Error;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::{Database, DeleteDataResponse, InsertDataResponse};

#[derive(Debug)]
pub struct PostgresDatabase {
    pub pool: PgPool,
}

impl PostgresDatabase {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Database for PostgresDatabase {
    async fn register_service(
        &self,
        service: &Service,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let uuid = Uuid::new_v4();
        match sqlx::query(
            "
        INSERT INTO services (id, name, endpoint)
        VALUES ($1, $2, $3)
        ",
        )
        .bind(uuid)
        .bind(service.name.clone())
        .bind(service.endpoint.clone())
        .execute(&self.pool)
        .await
        {
            Ok(result) => Ok(InsertDataResponse::Postgres(result.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn remove_service(
        &self,
        service_name: &str,
    ) -> Result<DeleteDataResponse, Box<dyn Error>> {
        match sqlx::query("DELETE services WHERE name = $1")
            .bind(service_name)
            .execute(&self.pool)
            .await
        {
            Ok(result) => Ok(DeleteDataResponse::Postgres(result.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn register_event_type(
        &self,
        event_type: &NewEventType,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let uuid = Uuid::new_v4();
        match sqlx::query(
            "
        INSERT INTO event_types (id, name, description, payload_desc)
        VALUES ($1, $2, $3, $4)
        ",
        )
        .bind(uuid)
        .bind(event_type.name.clone())
        .bind(event_type.description.clone())
        .bind(event_type.payload_description.clone())
        .execute(&self.pool)
        .await
        {
            Ok(result) => Ok(InsertDataResponse::Postgres(result.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn get_service(&self, name: &str) -> Result<Service, Box<dyn Error>> {
        match sqlx::query_as::<_, Service>(
            r#"
        SELECT name, endpoint as endpoint FROM services WHERE name = $1"#,
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        {
            Ok(service) => Ok(service),
            Err(err) => Err(Box::new(err)),
        }
    }
}
