use std::sync::Arc;

use provider_openai_compatible::OpenAiCompatibleService;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub openai_compatible: Arc<OpenAiCompatibleService>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let openai_compatible = Arc::new(OpenAiCompatibleService::new(
            config.openai_compatible.clone(),
        ));

        Self {
            config,
            openai_compatible,
        }
    }
}
