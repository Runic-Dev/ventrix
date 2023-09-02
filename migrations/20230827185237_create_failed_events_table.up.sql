-- Add up migration script here
CREATE TABLE failed_events (
    id UUID PRIMARY KEY,
    event_id UUID REFERENCES published_events (id),
    retry_time TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    retries SMALLINT DEFAULT 0,
    resolved_at TIMESTAMP DEFAULT NULL
)
