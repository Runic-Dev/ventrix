use serde_json::from_str;
use valico::json_schema;

pub fn payload_is_valid(payload: &String, schema: &String) -> bool {
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

#[cfg(test)]
pub mod test {

    use crate::schema_validator::payload_is_valid;

    #[test]
    pub fn passes_instance_with_valid_schema() {
        let payload: String = String::from(
            "
    {
        \"Address\": {
            \"Street\":\"Downing Street 10\",
            \"City\":\"London\",
            \"Country\":\"Great Britain\"
        }
    }
    ",
        );
        let schema = String::from(
            "
    {
        \"type\": \"object\",
        \"properties\": {
            \"Address\": {
                \"type\": \"object\",
                \"properties\": {
                    \"Country\": {
                        \"type\": \"string\"
                    },
                    \"Street\": {
                        \"type\": \"string\"
                    }
                },
                \"required\": [\"Country\", \"Street\"]
            }
        },
        \"required\": [\"Address\"]
    }",
        );

        assert!(payload_is_valid(&payload, &schema));
    }

    #[test]
    pub fn failes_instance_with_invalid_schema() {
        let payload: String = String::from(
            "
    {
        \"RandomPoint\": {
            \"Age\":\"69\",
            \"Object\": {
                \"This is an object\": 42
                },
            \"Bool\": false
        }
    }
    ",
        );
        let schema = String::from(
            "
    {
        \"type\": \"object\",
        \"properties\": {
            \"Address\": {
                \"type\": \"object\",
                \"properties\": {
                    \"Country\": {
                        \"type\": \"string\"
                    },
                    \"Street\": {
                        \"type\": \"string\"
                    }
                },
                \"required\": [\"Country\", \"Street\"]
            }
        },
        \"required\": [\"Address\"]
    }",
        );

        assert!(!payload_is_valid(&payload, &schema));
    }
}
