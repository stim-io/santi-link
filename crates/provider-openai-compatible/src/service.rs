use axum::{
    body::Bytes,
    http::{header, HeaderMap, HeaderValue},
};
use provider_openai_auth::{auth::OpenAiAuthEntry, AuthConfig, AuthService};
use reqwest::Client;
use tracing::error;

use crate::{config::OpenAiCompatibleConfig, models::UpstreamResponse};

#[derive(Debug, thiserror::Error)]
pub enum OpenAiCompatibleServiceError {
    #[error("failed to load openai auth: {0}")]
    Auth(#[from] provider_openai_auth::services::auth::AuthError),
    #[error("failed to send upstream request: {0}")]
    Request(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct OpenAiCompatibleService {
    config: OpenAiCompatibleConfig,
    auth: AuthService,
    client: Client,
}

impl OpenAiCompatibleService {
    pub fn new(config: OpenAiCompatibleConfig) -> Self {
        let auth = AuthService::new(AuthConfig {
            auth_file: config.auth_file.clone(),
            openai_client_id: config.openai_client_id.clone(),
            openai_issuer: config.openai_issuer.clone(),
        });

        Self {
            config,
            auth,
            client: Client::new(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.config.openai_compatible_api_endpoint
    }

    pub async fn post_responses(
        &self,
        request_headers: &HeaderMap,
        request_body: Bytes,
    ) -> Result<UpstreamResponse, OpenAiCompatibleServiceError> {
        let auth = self.auth.get_openai_auth().await?;
        let headers = upstream_headers(&auth, request_headers);

        let response = self
            .client
            .post(&self.config.openai_compatible_api_endpoint)
            .headers(headers)
            .body(request_body)
            .send()
            .await;

        let response = match response {
            Ok(response) => response,
            Err(error) => {
                error!(error = %error, endpoint = %self.config.openai_compatible_api_endpoint, "upstream request failed");
                return Err(OpenAiCompatibleServiceError::Request(error));
            }
        };

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await?;

        Ok(UpstreamResponse {
            status,
            headers: sanitize_response_headers(&headers),
            body,
        })
    }
}

fn upstream_headers(auth: &OpenAiAuthEntry, original: &HeaderMap) -> reqwest::header::HeaderMap {
    let mut next = reqwest::header::HeaderMap::new();

    for (name, value) in original {
        if name == header::HOST || name == header::CONTENT_LENGTH || name == header::AUTHORIZATION {
            continue;
        }

        if let Ok(converted_name) =
            reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes())
        {
            next.insert(converted_name, value.clone());
        }
    }

    next.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", auth.access))
            .unwrap_or_else(|_| HeaderValue::from_static("Bearer invalid")),
    );
    next.insert(
        reqwest::header::USER_AGENT,
        HeaderValue::from_static("providers/0.1.0"),
    );

    if let Some(account_id) = &auth.account_id {
        if let Ok(value) = HeaderValue::from_str(account_id) {
            next.insert(
                reqwest::header::HeaderName::from_static("chatgpt-account-id"),
                value,
            );
        }
    }

    next
}

fn sanitize_response_headers(original: &HeaderMap) -> HeaderMap {
    let mut next = HeaderMap::new();

    for (name, value) in original {
        if name == header::CONTENT_ENCODING || name == header::CONTENT_LENGTH {
            continue;
        }

        next.insert(name.clone(), value.clone());
    }

    next
}
