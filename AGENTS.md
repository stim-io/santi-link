# providers

## Provider Workspace

`providers/` is the provider workspace.

`providers/` is the upstream gateway layer inside the repo-root product and deployment boundary, while `santi/` owns core runtime semantics.

Do not publish real `auth.json`; keep local secrets in `auth.json` and use `auth.example.json` as the public template.

## Start Here

- `../AGENTS.md`: repo-root product and deployment boundary
- `docs/architecture.md`: current thin-proxy shape and boundaries

## Directory Rules

- `crates/api/src/{handlers,models}`: API host handlers, transport schemas, and route registration
- `crates/api/src/{config,state}.rs`: API host wiring and provider registration
- `crates/provider-openai-auth/src/{models,services}`: OpenAI auth models + AuthService
- `crates/provider-openai-compatible/src/{config,models,service}.rs`: OpenAI-compatible service contract and upstream forwarding
- `docs/`: architecture and cleanup notes
- `Dockerfile`: builds provider-api binary

## Common Commands (from `providers/`)

Run local Rust server:

```bash
cargo run -p provider-api
```

Build check:

```bash
cargo check -p provider-api
```

Format:

```bash
cargo fmt -p provider-api
```

Run the current `providers` Docker image:

```bash
docker build -t providers .
docker run --rm -p 8080:8080 -v "$PWD/auth.json:/app/auth.json" providers
```

Run Docker Compose (from root):

```bash
docker compose up --build
```

Smoke test current routes:

```bash
curl http://127.0.0.1:8080/openai/v1/health
curl -sN -X POST http://127.0.0.1:8080/openai/v1/responses \
  -H 'Content-Type: application/json' \
  --data '{"model":"gpt-5.4","instructions":"Reply with one short word.","input":[{"role":"user","content":[{"type":"input_text","text":"hello"}]}],"stream":true,"store":false}'
```

Current upstream contract requires `stream=true` and `store=false` for this smoke call.

## Runtime Defaults

- Swagger UI: `http://127.0.0.1:8080/swagger-ui`

Required local auth file shape: copy `auth.example.json` to `auth.json` and fill real credentials locally.

Useful env vars:

- `AUTH_FILE`: auth file path
- `PORT`: bind port
- `OPENAI_COMPATIBLE_API_ENDPOINT`: upstream OpenAI-compatible responses endpoint
- `OPENAI_CLIENT_ID`: OAuth client id for token refresh
- `OPENAI_ISSUER`: OAuth issuer base URL

## FAQ

Legacy provider source moved into `providers/`.
