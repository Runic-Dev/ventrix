-- Add up migration script here
CREATE TABLE event_types (
  id UUID PRIMARY KEY,
  name VARCHAR(255) UNIQUE NOT NULL,
  description VARCHAR(255),
  payload_desc VARCHAR(255)
)