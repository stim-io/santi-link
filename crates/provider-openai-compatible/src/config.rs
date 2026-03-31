#[derive(Debug, Clone)]
pub struct OpenAiCompatibleConfig {
    pub auth_file: String,
    pub openai_client_id: String,
    pub openai_issuer: String,
    pub openai_compatible_api_endpoint: String,
}
