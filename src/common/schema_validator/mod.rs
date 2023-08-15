use std::{error::Error, fmt::Display};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
use valico::json_schema;
use crate::common;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum JsonSchemaType {
    String
}

impl Display for JsonSchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            common::schema_validator::JsonSchemaType::String => write!(f, "string")
        }
    }
}

#[derive(Debug)]
struct InvalidPropertyTypeError {
    message: String
}

impl Display for InvalidPropertyTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InvalidPropertyTypeError {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonSchemaDef {
    property_type: JsonSchemaType,
    properties: Value,
    required: Vec<String>
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
        write!(f, "JsonSchemaDef with property type: {}, properties: {}, required: {}", self.property_type, properties, required)
    }
}

fn convert_event_type_to_schema_def(
    payload_definition: Value,
) -> Result<String, serde_json::Error> {
    let mut property_object = serde_json::Map::new();
    match payload_definition.as_object() {
        Some(property_map) => {
            for (key, _) in property_map.into_iter() {
                property_object.insert(
                    key.clone(),
                    json!({
                        "type": "string"
                    }),
                );
            }

            Ok(json!({
                "type": "object",
                "properties": property_object,
                "required": []
            })
            .to_string())
        }
        None => {
            print!("Here is what the received value is: {}", payload_definition);
            panic!();
        }
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    use crate::common::schema_validator::convert_event_type_to_schema_def;

    #[test]
    pub fn converts_empty_payload_def_to_empty_schema() -> Result<(), serde_json::Error> {
        let payload_definition = json!({});

        let schema = convert_event_type_to_schema_def(payload_definition)?;
        let expected_schema = json!({
                "type": "object",
                "properties": {},
                "required": []
        })
        .to_string();

        assert_eq!(schema, expected_schema);
        Ok(())
    }

    #[test]
    pub fn converts_payload_with_unrequired_string_field_to_correct_schema(
    ) -> Result<(), serde_json::Error> {
        let payload_definition = json!({
            "name": "John"
        });

        let schema = convert_event_type_to_schema_def(payload_definition)?;
        let expected_schema = json!({
            "type": "object",
            "properties": {
            "name": {
            "type": "string"
        }
        },
            "required": []
        })
        .to_string();

        assert_eq!(schema, expected_schema);
        Ok(())
    }



    #[test]
    pub fn converts_payload_with_required_string_field_to_correct_schema(
    ) -> Result<(), serde_json::Error> {
        let payload_definition = json!({
            "name": "John"
        });

        let schema = convert_event_type_to_schema_def(payload_definition)?;
        let expected_schema = json!({
            "type": "object",
            "properties": {
            "name": {
            "type": "string"
        }
        },
            "required": []
        })
        .to_string();

        assert_eq!(schema, expected_schema);
        Ok(())
    }
}
