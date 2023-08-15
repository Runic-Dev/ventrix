use std::future::{self, Ready};

use actix_web::{dev::Payload, FromRequest, HttpRequest};
use serde::Deserialize;

use crate::helpers::parsers::{parse_req_string, parse_req_uuid};

#[derive(Debug)]
#[derive(Eq, Hash, PartialEq)]
pub struct Service {
    pub id: uuid::Uuid,
    pub name: String,
    pub endpoint: String,
}

impl Service {
    fn parse_from_req(req: &HttpRequest) -> Result<Self, actix_web::Error> {
        let service_id = match parse_req_uuid(req, "service_id") {
            Ok(service_id) => service_id,
            Err(err) => {
                println!("Failed to parse service_id from request: {:?}", err);
                return Err(err);
            }
        };
        let service_name = match parse_req_string(req, "service_name") {
            Ok(service_name) => service_name,
            Err(err) => {
                println!("Failed to parse service_name from request: {:?}", err);
                return Err(err);
            }
        };
        let service_url = match parse_req_string(req, "service_url") {
            Ok(service_url) => service_url,
            Err(err) => {
                println!("Failed to parse service_url from request: {:?}", err);
                return Err(err);
            }
        };

        Ok(Service {
            id: service_id,
            name: service_name,
            endpoint: service_url,
        })
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

#[derive(Debug, Deserialize)]
pub struct RegisterServiceRequest {
    pub name: String,
    pub endpoint: String,
}

impl RegisterServiceRequest {
    fn parse_from_req(req: &HttpRequest) -> Result<Self, actix_web::Error> {
        let name = match parse_req_string(req, "name") {
            Ok(service_name) => service_name,
            Err(err) => {
                println!("Failed to parse service_name from request: {:?}", err);
                return Err(err);
            }
        };
        let endpoint = match parse_req_string(req, "endpoint") {
            Ok(service_url) => service_url,
            Err(err) => {
                println!("Failed to parse service_url from request: {:?}", err);
                return Err(err);
            }
        };

        Ok(RegisterServiceRequest { name, endpoint })
    }
}

impl FromRequest for RegisterServiceRequest {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let service = match Self::parse_from_req(req) {
            Ok(service) => service,
            Err(err) => {
                println!("Failed to parse service from request: {:?}", err);
                return future::ready(Err(err));
            }
        };
        future::ready(Ok(service))
    }
}
