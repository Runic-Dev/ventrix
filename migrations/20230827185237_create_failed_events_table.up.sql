-- Add up migration script here
CREATE TABLE failed_events (
    id UUID PRIMARY KEY,
    details JSON NOT NULL,
    retry_time TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    retries SMALLINT DEFAULT 0,
    resolved_at TIMESTAMP DEFAULT NULL
)
