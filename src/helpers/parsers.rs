use actix_web::HttpRequest;

pub fn parse_req_uuid(req: &HttpRequest, param_name: &str) -> Result<uuid::Uuid, actix_web::Error> {
    match req.match_info().get(param_name) {
        Some(param_str) => match uuid::Uuid::parse_str(param_str) {
            Ok(param) => Ok(param),
            Err(err) => {
                println!("Failed to parse {}: {:?}", param_name, err);
                Err(actix_web::error::ErrorBadRequest(format!(
                    "Failed to parse {}",
                    param_name
                )))
            }
        },
        None => {
            println!("{} is None", param_name);
            Err(actix_web::error::ErrorBadRequest(format!(
                "{} is None",
                param_name
            )))
        }
    }
}

pub fn parse_req_string(req: &HttpRequest, param_name: &str) -> Result<String, actix_web::Error> {
    match req.match_info().get(param_name) {
        Some(param_str) => Ok(String::from(param_str)),
        None => {
            println!("{} is None", param_name);
            Err(actix_web::error::ErrorBadRequest(format!(
                "{} is None",
                param_name
            )))
        }
    }
}

