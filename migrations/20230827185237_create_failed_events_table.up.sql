-- Add up migration script here
CREATE TABLE IF NOT EXISTS failed_events (
    id UUID PRIMARY KEY,
    event_id UUID REFERENCES events_published (id),
    retry_time TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    retries SMALLINT DEFAULT 0,
    resolved_at TIMESTAMPTZ DEFAULT NULL
)
