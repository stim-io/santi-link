use axum::{body::Bytes, http::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResponsesApiBody {
    #[schema(value_type = Object)]
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResponsesCreateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[schema(value_type = Object)]
    pub input: Value,
    pub stream: bool,
}

#[derive(Debug)]
pub struct UpstreamResponse {
    pub status: reqwest::StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}
