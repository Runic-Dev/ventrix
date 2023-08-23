-- Add up migration script here
CREATE TABLE event_type_to_service (
    id UUID PRIMARY KEY,
    event_type_id UUID,
    service_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
)
