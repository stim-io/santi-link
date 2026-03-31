use axum::{
    body::Bytes,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use provider_openai_compatible::ResponsesApiBody;
use serde_json::Value;
use tracing::{error, info};

use crate::{models::api_error::ApiErrorEnvelope, state::AppState};

#[utoipa::path(
    post,
    path = "/openai/v1/responses",
    tag = "providers",
    request_body(
        content = ResponsesApiBody,
        description = "OpenAI-compatible responses payload"
    ),
    responses(
        (status = 200, description = "Upstream responses payload"),
        (status = 400, description = "Invalid request body", body = ApiErrorEnvelope),
        (status = 500, description = "Server error", body = ApiErrorEnvelope)
    )
)]
pub async fn create_response_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if serde_json::from_slice::<Value>(&body).is_err() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(ApiErrorEnvelope::server_error(
                "request body must be valid json",
            )),
        )
            .into_response();
    }

    info!(
        endpoint = %state.openai_compatible.endpoint(),
        "proxying responses request"
    );

    match state.openai_compatible.post_responses(&headers, body).await {
        Ok(upstream) => (upstream.status, upstream.headers, upstream.body).into_response(),
        Err(error) => {
            error!(error = %error, "failed to proxy responses request");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(ApiErrorEnvelope::server_error(error.to_string())),
            )
                .into_response()
        }
    }
}
