use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct EventTypeAlreadyExistsError {
    pub message: String,
}

impl EventTypeAlreadyExistsError {
    pub fn new(event_type_name: String) -> Self {
        Self {
            message: format!("Event type: {:?} already exists", event_type_name),
        }
    }
}

impl Display for EventTypeAlreadyExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EventTypeAlreadyExistsError {}

#[derive(Debug)]
pub struct ServiceAlreadyExistsError {
    pub message: String,
}

impl ServiceAlreadyExistsError {
    pub fn new(service_name: String) -> Self {
        Self {
            message: format!("Service: {:?} already exists", service_name),
        }
    }
}

impl Display for ServiceAlreadyExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ServiceAlreadyExistsError {}
