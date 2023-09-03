use crate::common::types::{
    EventFulfillmentDetails, FailedEventRow, ListenToEventReq, PayloadSchema,
};
use crate::domain::models::service::RegisterServiceRequest;
use crate::infrastructure::persistence::NewEventTypeRequest;
use crate::{common::types::VentrixEvent, domain::models::service::Service};
use std::error::Error;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{Database, DeleteDataResponse, InsertDataResponse, UpdateDataResponse};

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
        service: &RegisterServiceRequest,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let uuid = Uuid::new_v4();
        match sqlx::query(
            "
        INSERT INTO services (id, name, url)
        VALUES ($1, $2, $3)
        ",
        )
        .bind(uuid)
        .bind(service.name.clone())
        .bind(service.url.clone())
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
        event_type: &NewEventTypeRequest,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let uuid = Uuid::new_v4();
        match sqlx::query(
            "
        INSERT INTO event_types (id, name, description, payload_definition)
        VALUES ($1, $2, $3, $4)
        ",
        )
        .bind(uuid)
        .bind(event_type.name.clone())
        .bind(event_type.description.clone())
        .bind(event_type.payload_definition.to_string())
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
        SELECT id, name, url FROM services WHERE name = $1"#,
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        {
            Ok(service) => Ok(service),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn save_published_event(
        &self,
        event: &VentrixEvent,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        match sqlx::query(
            "INSERT INTO events_published (id, event_type, payload) VALUES ($1, $2, $3)",
        )
        .bind(event.id)
        .bind(event.event_type.clone())
        .bind(event.payload.clone())
        .execute(&self.pool)
        .await
        {
            Ok(result) => Ok(InsertDataResponse::Postgres(result.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn add_failed_event(
        &self,
        event: &VentrixEvent,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let uuid = Uuid::new_v4();
        let retry_time = Utc::now() + Duration::minutes(1);
        match sqlx::query(
            "INSERT INTO failed_events (id, event_id, retry_time) VALUES ($1, $2, $3)",
        )
        .bind(uuid)
        .bind(event.id)
        .bind(retry_time)
        .execute(&self.pool)
        .await
        {
            Ok(response) => Ok(InsertDataResponse::Postgres(response.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn fulfil_event(
        &self,
        event: &VentrixEvent,
    ) -> Result<UpdateDataResponse, Box<dyn Error>> {
        match sqlx::query(r#"UPDATE events_published SET fulfilled_at = NOW() WHERE id = $1"#)
            .bind(event.id)
            .execute(&self.pool)
            .await
        {
            Ok(result) => Ok(UpdateDataResponse::Postgres(result.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn register_service_for_event_type(
        &self,
        listen_to_event_req: &ListenToEventReq,
    ) -> Result<InsertDataResponse, Box<dyn Error>> {
        let uuid = Uuid::new_v4();
        match sqlx::query(
            "WITH EventType AS (
                SELECT id FROM event_types WHERE name = $2
            ),
            Service AS (
                SELECT id FROM services WHERE name = $3
            )
            INSERT INTO event_type_to_service (id, event_type_id, service_id, endpoint)
            VALUES ($1, (SELECT id FROM EventType), (SELECT id FROM Service), $4)",
        )
        .bind(uuid)
        .bind(&listen_to_event_req.event_type.clone())
        .bind(&listen_to_event_req.service_name.clone())
        .bind(&listen_to_event_req.endpoint.clone())
        .execute(&self.pool)
        .await
        {
            Ok(result) => Ok(InsertDataResponse::Postgres(result.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn get_service_by_event_type(
        &self,
        event_type_name: &str,
    ) -> Result<Vec<EventFulfillmentDetails>, Box<dyn Error + Send>> {
        match sqlx::query_as::<_, EventFulfillmentDetails>(
            "SELECT services.name, services.url, event_type_to_service.endpoint 
            FROM services 
            INNER JOIN event_type_to_service ON event_type_to_service.service_id = services.id 
            INNER JOIN event_types ON event_type_to_service.event_type_id = event_types.id 
            WHERE event_types.name = $1;",
        )
        .bind(event_type_name.clone())
        .fetch_all(&self.pool)
        .await
        {
            Ok(event_fulfillment_details) => Ok(event_fulfillment_details),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn get_schema_for_event_type(
        &self,
        event_type_name: &str,
    ) -> Result<PayloadSchema, Box<dyn Error>> {
        match sqlx::query_as::<_, PayloadSchema>(
            r#"SELECT payload_definition FROM event_types WHERE name = $1"#,
        )
        .bind(event_type_name)
        .fetch_one(&self.pool)
        .await
        {
            Ok(schema) => Ok(schema),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn resolve_failed_event(
        &self,
        event_id: Uuid,
    ) -> Result<UpdateDataResponse, Box<dyn Error>> {
        match sqlx::query("UPDATE failed_events SET resolved_at = NOW() WHERE event_id = $1")
            .bind(event_id)
            .execute(&self.pool)
            .await
        {
            Ok(response) => Ok(UpdateDataResponse::Postgres(response.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn update_retry_time(
        &self,
        event_id: Uuid,
        new_retry_time: DateTime<Utc>,
        retries: i16
    ) -> Result<UpdateDataResponse, Box<dyn Error>> {
        match sqlx::query("UPDATE failed_events SET retry_time = $1, retries = $2 WHERE event_id = $3")
            .bind(new_retry_time)
            .bind(retries)
            .bind(event_id)
            .execute(&self.pool)
            .await
        {
            Ok(response) => Ok(UpdateDataResponse::Postgres(response.rows_affected())),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn get_failed_events(&self) -> Result<Vec<VentrixEvent>, Box<dyn Error + Sync + Send>> {
        match sqlx::query_as::<_, FailedEventRow>(
            r#"SELECT e.id, e.event_type, e.payload, f.retry_time, f.retries FROM events_published AS e INNER JOIN failed_events as f ON e.id = f.event_id WHERE f.retries < 3 AND f.retry_time < NOW()"#
        ).fetch_all(&self.pool).await {
                Ok(failed_event_rows) => {
                    Ok(failed_event_rows.iter().map(VentrixEvent::from_failed_event).collect())
                },
                Err(err) => Err(Box::new(err))
            }
    }
}
