use async_trait::async_trait;
use reqwest::{Client as HttpClient, Response};
use serde::de::Error;
use serde_json::{json, Value};
use urlencoding::encode;
use crate::errors::GatewayError;
use crate::models::{
    PaymentRequest, ChargeResponse, PaymentLinkResponse, RefundResponse, PaymentStatusResponse, WebhookResponse
};

async fn handle_stripe_response(response: Response) -> Result<Value, GatewayError> {
    let status = response.status();
    let body = response.json::<Value>().await?;
    if status.is_success() {
        Ok(body)
    } else {
        let message = body["error"]["message"].as_str()
            .unwrap_or("Unknown Stripe error")
            .to_string();
        Err(GatewayError::InvalidRequest(message))
    }
}

#[async_trait]
pub trait ChargeProcessor {
    async fn process_charge(&self, request: &PaymentRequest) -> Result<ChargeResponse, GatewayError>;
}

#[async_trait]
pub trait PaymentLinkProcessor {
    async fn process_payment_link(&self, request: &PaymentRequest) -> Result<PaymentLinkResponse, GatewayError>;
}

#[async_trait]
pub trait RefundProcessor {
    async fn process_refund(&self, request: &PaymentRequest) -> Result<RefundResponse, GatewayError>;
}

#[async_trait]
pub trait StatusProcessor {
    async fn process_status(&self, request: &PaymentRequest) -> Result<PaymentStatusResponse, GatewayError>;
}

#[async_trait]
pub trait WebhookProcessor {
    async fn process_webhook(&self, request: &PaymentRequest) -> Result<WebhookResponse, GatewayError>;
}

pub struct StripeChargeProcessor {
    http_client: HttpClient,
    api_key: String,
}

impl StripeChargeProcessor {
    pub fn new(api_key: String) -> Self {
        StripeChargeProcessor {
            http_client: HttpClient::new(),
            api_key,
        }
    }
}

#[async_trait]
impl ChargeProcessor for StripeChargeProcessor {
    async fn process_charge(&self, request: &PaymentRequest) -> Result<ChargeResponse, GatewayError> {
        log::info!("Processing charge for store: {}", request.store_id);
        if request.payment_token.is_none() {
            return Err(GatewayError::InvalidRequest("Payment token is required".to_string()));
        }
        let params = vec![
            ("amount", request.amount.unwrap_or(0).to_string()),
            ("currency", request.currency.as_deref().unwrap_or("").to_string()),
            ("source", request.payment_token.as_deref().unwrap_or("").to_string()),
            ("description", request.description.as_deref().unwrap_or("").to_string()),
        ];
        let form_data: String = params.iter()
            .map(|(k, v)| format!("{}={}", k, encode(v)))
            .collect::<Vec<String>>()
            .join("&");

        let response = self.http_client.post("https://api.stripe.com/v1/charges")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_data)
            .send()
            .await?;

        let body = handle_stripe_response(response).await?;

        Ok(ChargeResponse {
            status: "success".to_string(),
            message: None,
            charge_id: body["id"].as_str().map(String::from),
            amount: body["amount"].as_i64(),
            currency: body["currency"].as_str().map(String::from),
            status_code: 200,
        })
    }
}

pub struct StripePaymentLinkProcessor {
    http_client: HttpClient,
    api_key: String,
}

impl StripePaymentLinkProcessor {
    pub fn new(api_key: String) -> Self {
        StripePaymentLinkProcessor {
            http_client: HttpClient::new(),
            api_key,
        }
    }
}

#[async_trait]
impl PaymentLinkProcessor for StripePaymentLinkProcessor {
    async fn process_payment_link(&self, request: &PaymentRequest) -> Result<PaymentLinkResponse, GatewayError> {
        log::info!("Processing payment link for store: {}", request.store_id);
        if request.success_url.is_none() || request.cancel_url.is_none() {
            return Err(GatewayError::InvalidRequest("Success and cancel URLs are required".to_string()));
        }
        let params = json!({
            "mode": "payment",
            "line_items": [{
                "price_data": {
                    "currency": request.currency.as_deref().unwrap_or(""),
                    "unit_amount": request.amount.unwrap_or(0),
                    "product_data": {
                        "name": request.description.as_deref().unwrap_or("")
                    }
                },
                "quantity": 1
            }],
            "success_url": request.success_url.as_deref().unwrap_or(""),
            "cancel_url": request.cancel_url.as_deref().unwrap_or("")
        });

        let form_data: String = serde_urlencoded::to_string(&params)
            .map_err(|e| GatewayError::SerializationError(serde_json::Error::custom(e.to_string())))?;

        let response = self.http_client.post("https://api.stripe.com/v1/checkout/sessions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_data)
            .send()
            .await?;

        let body = handle_stripe_response(response).await?;

        Ok(PaymentLinkResponse {
            status: "success".to_string(),
            message: None,
            payment_link: body["url"].as_str().map(String::from),
            status_code: 200,
        })
    }
}

pub struct StripeRefundProcessor {
    http_client: HttpClient,
    api_key: String,
}

impl StripeRefundProcessor {
    pub fn new(api_key: String) -> Self {
        StripeRefundProcessor {
            http_client: HttpClient::new(),
            api_key,
        }
    }
}

#[async_trait]
impl RefundProcessor for StripeRefundProcessor {
    async fn process_refund(&self, request: &PaymentRequest) -> Result<RefundResponse, GatewayError> {
        log::info!("Processing refund for store: {}", request.store_id);
        if request.charge_id.is_none() {
            return Err(GatewayError::InvalidRequest("Charge ID is required".to_string()));
        }
        let params = json!({
            "charge": request.charge_id.as_deref().unwrap_or(""),
            "amount": request.amount.unwrap_or(0)
        });

        let form_data: String = serde_urlencoded::to_string(&params)
            .map_err(|e| GatewayError::SerializationError(serde_json::Error::custom(e.to_string())))?;

        let response = self.http_client.post("https://api.stripe.com/v1/refunds")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_data)
            .send()
            .await?;

        let body = handle_stripe_response(response).await?;

        Ok(RefundResponse {
            status: "success".to_string(),
            message: None,
            refund_id: body["id"].as_str().map(String::from),
            amount: body["amount"].as_i64(),
            currency: body["currency"].as_str().map(String::from),
            status_code: 200,
        })
    }
}

pub struct StripeStatusProcessor {
    http_client: HttpClient,
    api_key: String,
}

impl StripeStatusProcessor {
    pub fn new(api_key: String) -> Self {
        StripeStatusProcessor {
            http_client: HttpClient::new(),
            api_key,
        }
    }
}

#[async_trait]
impl StatusProcessor for StripeStatusProcessor {
    async fn process_status(&self, request: &PaymentRequest) -> Result<PaymentStatusResponse, GatewayError> {
        log::info!("Processing status check for store: {}", request.store_id);
        if request.charge_id.is_none() && request.session_id.is_none() {
            return Err(GatewayError::InvalidRequest("Charge ID or Session ID required".to_string()));
        }
        let (url, payment_id) = if let Some(charge_id) = &request.charge_id {
            (format!("https://api.stripe.com/v1/charges/{}", encode(charge_id)), charge_id.clone())
        } else {
            let session_id = request.session_id.as_deref().unwrap_or("");
            (format!("https://api.stripe.com/v1/checkout/sessions/{}", encode(session_id)), session_id.to_string())
        };
        let response = self.http_client.get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let body = handle_stripe_response(response).await?;

        Ok(PaymentStatusResponse {
            status: "success".to_string(),
            message: None,
            payment_id: Some(payment_id),
            payment_status: body["status"].as_str().map(String::from),
            amount: body["amount"].as_i64().or_else(|| body["amount_total"].as_i64()),
            currency: body["currency"].as_str().map(String::from),
            status_code: 200,
        })
    }
}

pub struct StripeWebhookProcessor {
    _http_client: HttpClient,
    _api_key: String,
}

impl StripeWebhookProcessor {
    pub fn new(api_key: String) -> Self {
        StripeWebhookProcessor {
            _http_client: HttpClient::new(),
            _api_key: api_key,
        }
    }
}

#[async_trait]
impl WebhookProcessor for StripeWebhookProcessor {
    async fn process_webhook(&self, request: &PaymentRequest) -> Result<WebhookResponse, GatewayError> {
        log::info!("Processing webhook for store: {}", request.store_id);
        if request.webhook_event.is_none() {
            return Err(GatewayError::InvalidRequest("Webhook event data required".to_string()));
        }
        let event = request.webhook_event.as_ref().unwrap();
        let event_id = event.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GatewayError::InvalidRequest("Webhook event ID required".to_string()))?;
        let event_type = event.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GatewayError::InvalidRequest("Webhook event type required".to_string()))?;

        log::debug!("Received webhook event: id={}, type={}", event_id, event_type);

        Ok(WebhookResponse {
            status: "success".to_string(),
            message: None,
            event_id: Some(event_id.to_string()),
            status_code: 200,
        })
    }
}