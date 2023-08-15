-- Add up migration script here
CREATE TABLE queue (
  id UUID PRIMARY KEY,
  event_type VARCHAR(255),
  payload VARCHAR(255),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
)
