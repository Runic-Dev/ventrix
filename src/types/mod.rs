use std::collections::VecDeque;

#[derive(Debug)]
pub struct VentrixEvent {
    pub event_type: String,
    pub payload: String,
}
pub type VentrixQueue = VecDeque<VentrixEvent>;
