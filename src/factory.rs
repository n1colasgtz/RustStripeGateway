use async_trait::async_trait;
use crate::errors::GatewayError;
use crate::models::PaymentRequest;
use crate::processors::{
    ChargeProcessor, PaymentLinkProcessor, RefundProcessor, StatusProcessor, WebhookProcessor,
    StripeChargeProcessor, StripePaymentLinkProcessor, StripeRefundProcessor, StripeStatusProcessor, StripeWebhookProcessor
};

#[async_trait]
pub trait PaymentProcessor {
    async fn process_payment(&self, request: &PaymentRequest) -> Result<serde_json::Value, GatewayError>;
}

pub struct PaymentProcessorFactory {
    api_key: String,
}

#[async_trait]
impl PaymentProcessor for PaymentProcessorFactory {
    async fn process_payment(&self, request: &PaymentRequest) -> Result<serde_json::Value, GatewayError> {
        match request.request_type.to_uppercase().as_str() {
            "CHARGE" => {
                let processor = StripeChargeProcessor::new(self.api_key.clone());
                let response = processor.process_charge(request).await?;
                Ok(serde_json::to_value(response)?)
            }
            "PAYMENT_LINK" => {
                let processor = StripePaymentLinkProcessor::new(self.api_key.clone());
                let response = processor.process_payment_link(request).await?;
                Ok(serde_json::to_value(response)?)
            }
            "REFUND" => {
                let processor = StripeRefundProcessor::new(self.api_key.clone());
                let response = processor.process_refund(request).await?;
                Ok(serde_json::to_value(response)?)
            }
            "STATUS" => {
                let processor = StripeStatusProcessor::new(self.api_key.clone());
                let response = processor.process_status(request).await?;
                Ok(serde_json::to_value(response)?)
            }
            "WEBHOOK" => {
                let processor = StripeWebhookProcessor::new(self.api_key.clone());
                let response = processor.process_webhook(request).await?;
                Ok(serde_json::to_value(response)?)
            }
            _ => Err(GatewayError::InvalidRequest(format!("Invalid request type: {}", request.request_type))),
        }
    }
}

impl PaymentProcessorFactory {
    pub fn new(api_key: String) -> Self {
        PaymentProcessorFactory { api_key }
    }
}