-- Add up migration script here
CREATE TABLE IF NOT EXISTS events_published (
  id UUID PRIMARY KEY,
  event_type VARCHAR(255),
  payload VARCHAR(255),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  fulfilled_at TIMESTAMPTZ
)
