use actix_web::http::HeaderMap;
use actix_web::web::{BytesMut, Payload};
use actix_web::HttpRequest;
use futures::StreamExt;
use std::collections::HashMap;
use std::net::IpAddr;

use crate::core::request::ThrusterRequest;

pub struct ActixRequest {
    pub path: String,
    pub method: String,
    pub headers: HeaderMap,
    pub payload: Vec<u8>,
    pub params: Option<HashMap<String, String>>,
    pub ip: Option<IpAddr>,
}

impl ActixRequest {
    pub async fn new(request: HttpRequest, mut payload: Payload) -> ActixRequest {
        let path = request.path().to_string();
        let headers = request.headers().clone();
        let method = request.method().to_string();

        let mut bytes = BytesMut::new();
        while let Some(item) = payload.next().await {
            bytes.extend_from_slice(&item.unwrap());
        }

        ActixRequest {
            path,
            method,
            payload: bytes.to_vec(),
            headers,
            params: None,
            ip: None,
        }
    }
}

impl ThrusterRequest for ActixRequest {
    fn method(&self) -> &str {
        ""
    }

    fn path(&self) -> String {
        self.path.to_string()
    }
}
