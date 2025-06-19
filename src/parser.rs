use serde_json::{Value, from_value};
use crate::errors::GatewayError;
use crate::models::PaymentRequest;

pub struct JsonRequestParser;

impl JsonRequestParser {
    pub fn new() -> Self {
        JsonRequestParser
    }

    pub fn parse(&self, input: Value) -> Result<PaymentRequest, GatewayError> {
        let body = input.get("body")
            .ok_or_else(|| GatewayError::InvalidRequest("Request body is missing".to_string()))?;

        if body.is_string() {
            let body_str = body.as_str().unwrap();
            from_value(serde_json::from_str(body_str)?)
                .map_err(|e| GatewayError::SerializationError(e))
        } else if body.is_object() {
            from_value(body.clone())
                .map_err(|e| GatewayError::SerializationError(e))
        } else {
            Err(GatewayError::InvalidRequest("Body must be a JSON string or object".to_string()))
        }
    }
}