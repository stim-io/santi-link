pub mod config;
pub mod handlers;
pub mod models;
pub mod state;

use axum::{
    routing::{get, post},
    Router,
};
use provider_openai_compatible::{ResponsesApiBody, ResponsesCreateRequest};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::health::HealthResponse;
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health_handler,
        crate::handlers::responses::create_response_handler
    ),
    components(schemas(
        HealthResponse,
        ResponsesApiBody,
        ResponsesCreateRequest
    )),
    tags((name = "providers", description = "API host for provider-backed upstream responses"))
)]
pub struct ApiDoc;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route(
            "/openai/v1/health",
            get(crate::handlers::health::health_handler),
        )
        .route(
            "/openai/v1/responses",
            post(crate::handlers::responses::create_response_handler),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
