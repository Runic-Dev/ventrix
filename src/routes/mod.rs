pub mod events;
mod health_check;
pub mod queue;
pub mod services;

pub use health_check::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeleteServiceRequest {
    pub name: String
}
