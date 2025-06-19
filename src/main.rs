use lambda_runtime::{run, service_fn, Error as LambdaError, LambdaEvent};
use serde_json::{json, Value};
use log::{info, error};
use env_logger::Env;
use crate::services::SecretsService;
use crate::parser::JsonRequestParser;
use crate::models::{PaymentRequest, ErrorResponse};
use crate::factory::{PaymentProcessorFactory, PaymentProcessor};
use crate::errors::GatewayError;

mod errors;
mod models;
mod services;
mod parser;
mod processors;
mod factory;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, LambdaError> {
    info!("Received event: {:?}", event);
    let parser = JsonRequestParser::new();
    let request = match parser.parse(event.payload) {
        Ok(req) => req,
        Err(e) => return Ok(json!({
            "statusCode": 400,
            "body": ErrorResponse {
                status: "error".to_string(),
                message: format!("Invalid request: {}", e),
                status_code: 400,
            }
        })),
    };

    let secrets_service = match SecretsService::new().await {
        Ok(service) => service,
        Err(e) => return Ok(json!({
            "statusCode": 500,
            "body": ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to initialize secrets service: {}", e),
                status_code: 500,
            }
        })),
    };

    let api_key = match secrets_service.get_secret(&request.store_id).await {
        Ok(key) => key,
        Err(e) => return Ok(json!({
            "statusCode": 500,
            "body": ErrorResponse {
                status: "error".to_string(),
                message: format!("Failed to retrieve API key: {}", e),
                status_code: 500,
            }
        })),
    };

    let factory = PaymentProcessorFactory::new(api_key);
    match factory.process_payment(&request).await {
        Ok(response) => Ok(json!({
            "statusCode": response["statusCode"].as_i64().unwrap_or(500),
            "body": response
        })),
        Err(e) => {
            error!("Error processing request: {}", e);
            Ok(json!({
                "statusCode": 500,
                "body": ErrorResponse {
                    status: "error".to_string(),
                    message: e.to_string(),
                    status_code: 500,
                }
            }))
        }
    }
}