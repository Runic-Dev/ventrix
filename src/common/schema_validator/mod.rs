use crate::common;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Map, Value};
use std::{fmt::Display, str::FromStr};
use valico::{
    json_dsl::{self, Builder},
    json_schema::{self},
};

use super::{errors::InvalidPropertyDef, types::NewEventTypeRequest};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum JsonSchemaType {
    String,
}

// Payload coming from published event
pub fn payload_is_valid(payload: &str, schema: &str) -> bool {
    let payload_as_obj = match from_str(payload) {
        Ok(payload_obj) => payload_obj,
        Err(err) => {
            tracing::warn!("Could not parse payload object: {:?}", err);
            return false;
        }
    };
    let schema_as_obj = match from_str(schema) {
        Ok(schema_obj) => schema_obj,
        Err(err) => {
            tracing::warn!("Could not parse schema object: {:?}", err);
            return false;
        }
    };

    let mut scope = json_schema::Scope::new();
    let r_schema = scope.compile_and_return(schema_as_obj, true).ok().unwrap();

    r_schema.validate(&payload_as_obj).is_valid()
}

pub fn create_schema_from_request(request: NewEventTypeRequest) {
    let payload_def = request.payload_definition;
}

impl Display for JsonSchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            common::schema_validator::JsonSchemaType::String => write!(f, "string"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonSchemaDef {
    property_type: JsonSchemaType,
    properties: Value,
    required: Vec<String>,
}

impl Display for JsonSchemaDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut properties = String::from("");

        if self.properties.is_string() {
            properties.push_str(&self.properties.to_string());
        } else if self.properties.is_object() {
            let map = self.properties.as_object().unwrap();
            for (key, value) in map.into_iter() {
                properties.push_str(&format!("key: {}, value: {}", key, value));
            }
        }

        let mut required = String::from("");

        for req in self.required.iter() {
            required.push_str(req);
        }
        write!(
            f,
            "JsonSchemaDef with property type: {}, properties: {}, required: {}",
            self.property_type, properties, required
        )
    }
}

fn get_properties_as_map(value: &mut Value) -> Result<Map<String, Value>, InvalidPropertyDef> {
    match value["properties"].as_object() {
        Some(parsed_props) => Ok(parsed_props.clone()),
        None => Err(InvalidPropertyDef::new(String::from(
            "Properties should be an object",
        ))),
    }
}

#[derive(PartialEq, Eq)]
enum SchemaProperty {
    String,
    Object,
    Number,
}

impl FromStr for SchemaProperty {
    type Err = InvalidPropertyDef;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_case = s.to_lowercase();
        let s = lower_case.as_str();
        match s {
            "string" => Ok(SchemaProperty::String),
            "object" => Ok(SchemaProperty::Object),
            "number" => Ok(SchemaProperty::Number),
            _ => Err(InvalidPropertyDef::new(String::from(s))),
        }
    }
}

impl Display for SchemaProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaProperty::String => write!(f, "String"),
            SchemaProperty::Object => write!(f, "Object"),
            SchemaProperty::Number => write!(f, "Number"),
        }
    }
}

impl SchemaProperty {
    fn get_verico_params(&self) -> Builder {
        match self {
            SchemaProperty::String => Builder::build(|params| {
                params.req_typed("type", json_dsl::string());
            }),
            SchemaProperty::Object => Builder::build(|params| {
                params.req_typed("type", json_dsl::string());
                params.req_typed("properties", json_dsl::object());
                params.req_typed("required", json_dsl::array_of(json_dsl::string()));
            }),
            SchemaProperty::Number => {
                Builder::build(|params| params.req_typed("type", json_dsl::string()))
            }
        }
    }
}

pub fn is_valid_property_def(value: &mut Value) -> Result<Value, InvalidPropertyDef> {
    let mut properties = get_properties_as_map(value)?;

    for (key, value) in properties.iter_mut() {
        let property_details = value.as_object().ok_or_else(|| {
            InvalidPropertyDef::new(format!("Definition for property {} is invalid", key))
        })?;

        let property_type = property_details
            .get("type")
            .ok_or_else(|| InvalidPropertyDef {
                message: format!("Type for property {} not defined correctly", key),
            })?;

        let property_type_str = property_type.as_str().ok_or_else(|| InvalidPropertyDef {
            message: format!("Type for property {} must be a string", key),
        })?;

        let prop_type = SchemaProperty::from_str(property_type_str)?;

        let params = prop_type.get_verico_params();

        if !params.process(value, None).is_strictly_valid() {
            return Err(InvalidPropertyDef::new(format!(
                "Definition for property {} is invalid for a {} type",
                key, prop_type
            )));
        }

        if prop_type == SchemaProperty::Object {
            return is_valid_property_def(value);
        }
    }

    Ok(value.clone())
}

// Create a schema

#[cfg(test)]
pub mod tests {
    use serde_json::json;

    use crate::common::types::NewEventTypeRequest;

    use super::is_valid_property_def;

    #[test]
    pub fn should_validate_valid_definition() {
        let mut request = NewEventTypeRequest {
            name: String::from("test_event"),
            description: String::from("This is a test event"),
            payload_definition: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string"
                    },
                    "age": {
                        "type": "number"
                    }
                },
                "required": []
            }),
        };

        is_valid_property_def(&mut request.payload_definition).unwrap();
    }

    #[test]
    #[should_panic]
    pub fn should_invalidate_invalid_definition() {
        let mut request = NewEventTypeRequest {
            name: String::from("test_event"),
            description: String::from("This is a test event"),
            payload_definition: json!({
                "type": "object",
                "properties": {
                    "name": "this is invalid",
                    "age": {
                        "type": "number"
                    }
                },
                "required": []
            }),
        };

        is_valid_property_def(&mut request.payload_definition).unwrap();
    }

    #[test]
    pub fn should_save_a_schema_from_new_event_type_req() {
        let mut request = NewEventTypeRequest {
            name: String::from("test_event"),
            description: String::from("This is a test event"),
            payload_definition: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string"
                    },
                    "age": {
                        "type": "number"
                    }
                },
                "required": []
            }),
        };

        let validated_payload_def = is_valid_property_def(&mut request.payload_definition).unwrap();
        let as_string = validated_payload_def.to_string();
    }
}
