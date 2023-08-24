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

#[derive(Debug)]
pub struct EventNotFoundError {
    pub message: String,
}

impl EventNotFoundError {
    pub fn new(id: &str) -> Self {
        Self {
            message: format!("Event: {:?} not found", id),
        }
    }
}

impl Error for EventNotFoundError {}

impl Display for EventNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
#[derive(Debug)]
pub struct EventTypeNotFoundError {
    pub message: String,
}

impl EventTypeNotFoundError {
    pub fn new(id: &str) -> Self {
        Self {
            message: format!("EventType: {:?} not found", id),
        }
    }
}

impl Error for EventTypeNotFoundError {}

impl Display for EventTypeNotFoundError {
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

#[derive(Debug)]
pub struct InvalidPropertyDef {
    pub message: String,
}

impl InvalidPropertyDef {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Error for InvalidPropertyDef {}

impl Display for InvalidPropertyDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
struct InvalidPropertyTypeError {
    message: String,
}

impl Display for InvalidPropertyTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InvalidPropertyTypeError {}
