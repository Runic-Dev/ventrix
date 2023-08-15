use std::fmt::Display;

use log::debug;
use serde::Deserialize;
use serde_json::{from_str, json, Value};
use valico::json_schema;

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
}
