use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct PaymentRequest {
    #[serde(rename = "storeId")]
    pub store_id: String,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    #[serde(rename = "paymentToken")]
    pub payment_token: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "requestType")]
    pub request_type: String,
    #[serde(rename = "successUrl")]
    pub success_url: Option<String>,
    #[serde(rename = "cancelUrl")]
    pub cancel_url: Option<String>,
    #[serde(rename = "chargeId")]
    pub charge_id: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "webhookEvent")]
    pub webhook_event: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize, Debug)]
pub struct ChargeResponse {
    pub status: String,
    pub message: Option<String>,
    #[serde(rename = "chargeId")]
    pub charge_id: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
}

#[derive(Serialize, Debug)]
pub struct PaymentLinkResponse {
    pub status: String,
    pub message: Option<String>,
    #[serde(rename = "paymentLink")]
    pub payment_link: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
}

#[derive(Serialize, Debug)]
pub struct RefundResponse {
    pub status: String,
    pub message: Option<String>,
    #[serde(rename = "refundId")]
    pub refund_id: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
}

#[derive(Serialize, Debug)]
pub struct PaymentStatusResponse {
    pub status: String,
    pub message: Option<String>,
    #[serde(rename = "paymentId")]
    pub payment_id: Option<String>,
    #[serde(rename = "paymentStatus")]
    pub payment_status: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
}

#[derive(Serialize, Debug)]
pub struct WebhookResponse {
    pub status: String,
    pub message: Option<String>,
    #[serde(rename = "eventId")]
    pub event_id: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
}