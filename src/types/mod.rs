use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VentrixEvent {
    pub event_type: String,
    pub payload: String,
}
