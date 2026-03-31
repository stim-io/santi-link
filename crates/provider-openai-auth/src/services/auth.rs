use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::Client;
use tokio::{fs, sync::Mutex};

use crate::models::auth::{AuthFile, OAuthRefreshResponse, OpenAiAuthEntry};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("auth file not found: {0}")]
    AuthFileNotFound(String),
    #[error("failed to read auth file: {0}")]
    Read(#[from] std::io::Error),
    #[error("failed to parse auth file: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("auth.json must contain openai oauth credentials")]
    InvalidAuthType,
    #[error("auth.json is missing access/refresh tokens")]
    MissingTokens,
    #[error("token refresh failed: HTTP {0}")]
    RefreshFailed(reqwest::StatusCode),
    #[error("failed to decode refresh response: {0}")]
    RefreshDecode(#[source] reqwest::Error),
    #[error("failed to send refresh request: {0}")]
    RefreshRequest(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub auth_file: String,
    pub openai_client_id: String,
    pub openai_issuer: String,
}

#[derive(Debug)]
pub struct AuthService {
    config: AuthConfig,
    client: Client,
    write_lock: Mutex<()>,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            write_lock: Mutex::new(()),
        }
    }

    pub async fn get_openai_auth(&self) -> Result<OpenAiAuthEntry, AuthError> {
        let auth_file = self.read_auth_file().await?;

        if self.needs_refresh(&auth_file.openai) {
            let refreshed = self.refresh_auth(auth_file).await?;
            return Ok(refreshed.openai);
        }

        Ok(auth_file.openai)
    }

    async fn read_auth_file(&self) -> Result<AuthFile, AuthError> {
        let path = PathBuf::from(&self.config.auth_file);

        if !path.exists() {
            return Err(AuthError::AuthFileNotFound(self.config.auth_file.clone()));
        }

        let raw = fs::read_to_string(path).await?;
        let auth_file: AuthFile = serde_json::from_str(&raw)?;

        if auth_file.openai.auth_type != "oauth" {
            return Err(AuthError::InvalidAuthType);
        }

        if auth_file.openai.refresh.trim().is_empty() || auth_file.openai.access.trim().is_empty() {
            return Err(AuthError::MissingTokens);
        }

        Ok(auth_file)
    }

    fn needs_refresh(&self, auth: &OpenAiAuthEntry) -> bool {
        match auth.expires {
            Some(expires) => expires <= now_millis(),
            None => true,
        }
    }

    async fn refresh_auth(&self, mut auth_file: AuthFile) -> Result<AuthFile, AuthError> {
        let _guard = self.write_lock.lock().await;

        let latest = self.read_auth_file().await?;
        if !self.needs_refresh(&latest.openai) {
            return Ok(latest);
        }

        let response = self
            .client
            .post(format!("{}/oauth/token", self.config.openai_issuer))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", latest.openai.refresh.as_str()),
                ("client_id", self.config.openai_client_id.as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AuthError::RefreshFailed(response.status()));
        }

        let payload = response
            .json::<OAuthRefreshResponse>()
            .await
            .map_err(AuthError::RefreshDecode)?;

        auth_file.openai.access = payload.access_token;
        if let Some(refresh_token) = payload.refresh_token {
            auth_file.openai.refresh = refresh_token;
        }
        auth_file.openai.expires = payload
            .expires_in
            .map(|seconds| now_millis() + seconds * 1000);

        let raw = serde_json::to_string_pretty(&auth_file)?;
        fs::write(&self.config.auth_file, raw).await?;

        Ok(auth_file)
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
