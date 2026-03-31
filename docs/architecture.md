# Architecture (providers)

`providers/` owns the thin auth proxy for OpenAI/Codex Responses.

## Source Layout

- `crates/api/src/handlers`: API host handlers and route entrypoints
- `crates/api/src/{config,state}.rs`: API host config and provider registration
- `crates/provider-openai-auth/src/services/auth.rs`: OAuth auth loading/refresh
- `crates/provider-openai-auth/src/models/auth.rs`: auth schemas
- `crates/provider-openai-compatible/src/{config,models,service}.rs`: OpenAI-compatible config, request models, and upstream service

## Current Status

- `GET /openai/v1/health` is implemented
- `POST /openai/v1/responses` is implemented as the main proxy path
- `auth.json` loading and OAuth refresh are implemented in the auth crate
- `auth.example.json` is the public template; real `auth.json` stays local
- upstream auth injection is implemented through `Authorization` and `chatgpt-account-id`
- OpenAPI is generated with `utoipa`
- Swagger UI is mounted at `/swagger-ui`
- container delivery runs the provider-api binary on port `8080`

## Boundary

The API host keeps only transport and registration concerns:

- route registration
- handler declaration
- OpenAPI / Swagger wiring

Provider-specific service crates keep upstream account-facing concerns:

- OAuth credential loading and refresh (provider-openai-auth)
- upstream auth/header injection
- basic forwarding and header hygiene

The server does not own:

- `previous_response_id` rewriting
- response cache for continuation semantics
- chat/responses conversion
- tool-call extraction
- SSE semantic parsing
- prompt cache key generation

Those protocol responsibilities belong in the caller-side provider adapter.
