use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Stripe API error: {0}")]
    StripeError(#[from] reqwest::Error),
    #[error("Secrets Manager error: {0}")]
    SecretsManagerError(#[from] aws_sdk_secretsmanager::error::SdkError<aws_sdk_secretsmanager::operation::get_secret_value::GetSecretValueError>),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}