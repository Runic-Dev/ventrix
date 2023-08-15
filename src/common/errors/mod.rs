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

#[derive(Debug)]
pub struct ServiceNotFoundError {
    pub message: String,
}

impl ServiceNotFoundError {
    pub fn new(name: &str) -> Self {
        Self {
            message: format!("Service: {:?} not found", name),
        }
    }
}

impl Display for ServiceNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ServiceNotFoundError {}

#[derive(Debug)]
pub struct ParsingRecordToStructError {
    pub message: String,
}

impl ParsingRecordToStructError {
    pub fn new(struct_name: String) -> Self {
        Self {
            message: format!("Could not parse record into struct: {:?}", struct_name),
        }
    }
}

impl Error for ParsingRecordToStructError {}

impl Display for ParsingRecordToStructError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
