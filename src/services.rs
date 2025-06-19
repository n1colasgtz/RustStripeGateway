use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use serde_json::Value;
use crate::errors::GatewayError;

pub struct SecretsService {
    client: SecretsManagerClient,
}

impl SecretsService {
    pub async fn new() -> Result<Self, GatewayError> {
        let config = aws_config::load_from_env().await;
        let client = SecretsManagerClient::new(&config);
        Ok(SecretsService { client })
    }

    pub async fn get_secret(&self, secret_id: &str) -> Result<String, GatewayError> {
        if secret_id.is_empty() {
            return Err(GatewayError::InvalidRequest("Store ID cannot be empty".to_string()));
        }
        let response = self.client.get_secret_value()
            .secret_id(secret_id)
            .send()
            .await?;

        let secret_string = response.secret_string
            .ok_or_else(|| GatewayError::Unexpected(format!("Secret is empty for store: {}", secret_id)))?;

        // Try parsing as JSON to extract stripeSecretKey
        if let Ok(json) = serde_json::from_str::<Value>(&secret_string) {
            if let Some(api_key) = json.get("stripeSecretKey").and_then(|v| v.as_str()) {
                if !api_key.trim().is_empty() {
                    return Ok(api_key.to_string());
                }
            }
        }

        if secret_string.trim().is_empty() {
            return Err(GatewayError::Unexpected(format!("Secret is empty for store: {}", secret_id)));
        }
        Ok(secret_string)
    }
}