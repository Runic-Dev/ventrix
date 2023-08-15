use std::future::{Ready, self};

use actix_web::{FromRequest, HttpRequest, dev::Payload};

#[derive(Debug)]
pub struct Service {
    pub id: uuid::Uuid,
    pub name: String,
    pub url: String,
}

impl Service {
    fn parse_from_req(req: &HttpRequest) -> Result<Self, actix_web::Error> {
        let service_id = match Self::parse_req_uuid(req, "service_id") {
            Ok(service_id) => service_id,
            Err(err) => {
                println!("Failed to parse service_id from request: {:?}", err);
                return Err(err);
            }
        };
        let service_name = match Self::parse_req_string(req, "service_name") {
            Ok(service_name) => service_name,
            Err(err) => {
                println!("Failed to parse service_name from request: {:?}", err);
                return Err(err);
            }
        };
        let service_url = match Self::parse_req_string(req, "service_url") {
            Ok(service_url) => service_url,
            Err(err) => {
                println!("Failed to parse service_url from request: {:?}", err);
                return Err(err);
            }
        };

        Ok(Service {
            id: service_id,
            name: service_name,
            url: service_url,
        })
    }

    fn parse_req_uuid(req: &HttpRequest, param_name: &str) -> Result<uuid::Uuid, actix_web::Error> {
        match req.match_info().get(param_name) {
            Some(param_str) => match uuid::Uuid::parse_str(param_str) {
                Ok(param) => Ok(param),
                Err(err) => {
                    println!("Failed to parse {}: {:?}", param_name, err);
                    Err(actix_web::error::ErrorBadRequest(format!("Failed to parse {}", param_name)))
                }
            }
            None => {
                println!("{} is None", param_name);
                Err(actix_web::error::ErrorBadRequest(format!("{} is None", param_name)))
            }
        }
    }

    fn parse_req_string(req: &HttpRequest, param_name: &str) -> Result<String, actix_web::Error> {
        match req.match_info().get(param_name) {
            Some(param_str) => Ok(String::from(param_str)),
            None => {
                println!("{} is None", param_name);
                Err(actix_web::error::ErrorBadRequest(format!("{} is None", param_name)))
            }
        }
    }
}

impl FromRequest for Service {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let service = match Service::parse_from_req(req) {
            Ok(service) => service,
            Err(err) => {
                println!("Failed to parse service from request: {:?}", err);
                return future::ready(Err(err));
            }
        };
        future::ready(Ok(service))
    }
}
