use crate::common;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Map, Value};
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

fn get_properties_as_map(value: &mut Value) -> Result<Map<String, Value>, InvalidPropertyDef> {
    match value["properties"].as_object() {
        Some(parsed_props) => Ok(parsed_props.clone()),
        None => Err(InvalidPropertyDef::new(String::from(
            "Properties should be an object",
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

enum SchemaProperty {
    String,
    Object,
    _Number,
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

impl Display for SchemaProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaProperty::String => write!(f, "String"),
            SchemaProperty::Object => write!(f, "Object"),
            SchemaProperty::_Number => write!(f, "Number"),
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
            SchemaProperty::_Number => Builder::build(|_params| todo!()),
        }
    }
}

pub fn is_valid_property_def(value: &mut Value) -> Result<(), InvalidPropertyDef> {
    let mut properties = get_properties_as_map(value)?;

    for (key, value) in properties.iter_mut() {
        let property_details = value.as_object().ok_or_else(|| {
            InvalidPropertyDef::new(format!("Definition for property {} is invalid", key))
        })?;

        let prop_type_as_string = get_property_type_as_str(property_details, key)?;

        let prop_type = SchemaProperty::from_str(prop_type_as_string)?;

        let params = prop_type.get_verico_params();

        if !params.process(value, None).is_strictly_valid() {
            return Err(InvalidPropertyDef::new(format!(
                "Definition for property {} is invalid for a {} type",
                key, prop_type
            )));
        }
    }

    Ok(())
}
