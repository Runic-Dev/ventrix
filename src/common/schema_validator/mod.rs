use crate::common;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Map, Value};
use std::{error::Error, fmt::Display, str::FromStr};
use valico::{
    json_dsl::{self, Builder},
    json_schema::{self},
};

use super::errors::InvalidPropertyDef;

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
    String,
}

impl Display for JsonSchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            common::schema_validator::JsonSchemaType::String => write!(f, "string"),
        }
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

fn is_valid_event_def(value: &mut Value) -> bool {
    let params = Builder::build(|params| {
        params.req_typed("type", json_dsl::string());
        params.req_typed("properties", json_dsl::object());
        params.req_typed("required", json_dsl::array());
    });

    params.process(value, None).is_strictly_valid()
}

fn get_properties_as_map(value: &mut Value) -> Result<Map<String, Value>, InvalidPropertyDef> {
    match value["properties"].as_object() {
        Some(parsed_props) => Ok(parsed_props.clone()),
        None => Err(InvalidPropertyDef::new(String::from(
            "Properties should be an object",
        ))),
    }
}

fn get_properties_details_as_map<'a>(
    value: &'a Value,
    key: &'a String,
) -> Result<&'a Map<String, Value>, InvalidPropertyDef> {
    match value["properties"].as_object() {
        Some(val_obj) => Ok(val_obj),
        None => Err(InvalidPropertyDef::new(format!(
            "Definition for {} is not an object",
            key
        ))),
    }
}

fn get_property_type_as_str<'a>(
    property_details: &'a Map<String, Value>,
    key: &'a String,
) -> Result<&'a str, InvalidPropertyDef> {
    match property_details.get("type") {
        Some(prop_type) => {
            let prop_type_as_string = match prop_type.as_str() {
                Some(string_value) => string_value,
                None => {
                    return Err(InvalidPropertyDef::new(format!(
                        "Type for {} is not a string",
                        key
                    )))
                }
            };
            Ok(prop_type_as_string)
        }
        None => Err(InvalidPropertyDef::new(format!(
            "Type was not defined for property {}",
            key
        ))),
    }
}

fn check_string_property_def(value: &mut Value, key: &str) -> Result<(), InvalidPropertyDef> {
    let params = Builder::build(|params| {
        params.req_typed("type", json_dsl::string());
    });
    if params.process(value, None).is_strictly_valid() {
        return Ok(());
    }
    Err(InvalidPropertyDef::new(format!(
        "Definition for property {} is invalid for a string type",
        key
    )))
}

fn check_object_property_def(value: &mut Value, key: &str) -> Result<(), InvalidPropertyDef> {
    let params = Builder::build(|params| {
        params.req_typed("type", json_dsl::string());
        params.req_typed("properties", json_dsl::object());
        params.req_typed("required", json_dsl::array_of(json_dsl::string()));
    });

    if params.process(value, None).is_strictly_valid() {
        return Ok(());
    }
    Err(InvalidPropertyDef::new(format!(
        "Definition for property {} is invalid for a object type",
        key
    )))
}

enum SchemaProperty {
    String,
    Object,
}

impl FromStr for SchemaProperty {
    type Err = InvalidPropertyDef;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_case = s.to_lowercase();
        let s = lower_case.as_str();
        match s {
            "string" => Ok(SchemaProperty::String),
            "object" => Ok(SchemaProperty::Object),
            _ => Err(InvalidPropertyDef::new(String::from(s))),
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
        }
    }
}

fn is_valid_property_def(value: &mut Value) -> Result<(), InvalidPropertyDef> {
    let mut properties = get_properties_as_map(value)?;

    for (key, value) in properties.iter_mut() {
        let value_clone = &value.clone();
        let property_details = get_properties_details_as_map(value_clone, key)?;

        let prop_type_as_string = get_property_type_as_str(property_details, key)?;

        let prop_type = SchemaProperty::from_str(prop_type_as_string)?;

        let params = prop_type.get_verico_params();

        if !params.process(value, None).is_strictly_valid() {
            return Err(InvalidPropertyDef::new(format!(
                "Definition for property {} is invalid for a {} type",
                key, prop_type_as_string
            )));
        }
    }

    Ok(())
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
