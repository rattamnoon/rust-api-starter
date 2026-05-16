# Rust API Starter

Rust backend starter for building web APIs with:
- Actix Web
- SQLx
- PostgreSQL
- JWT authentication
- Swagger/OpenAPI
- structured logging
- rate limiting
- local file uploads

This repository is set up as a modular service with clear separation between handler, service, repository, model, and DTO layers.

## Project Structure
- `src/`: application code
- `migrations/`: SQLx database migrations
- `docs/`: engineering knowledge base and architecture docs
- `logs/`: runtime log files

## Quick Start
1. Start PostgreSQL only:
   ```bash
   docker compose up -d postgres
   ```
2. Create local environment file:
   ```bash
   cp .env.example .env
   ```
3. Run the API:
   ```bash
   cargo run --bin rust-api-starter
   ```

## Docker Deploy
Build and run the full stack with Docker Compose:
```bash
docker compose -f docker-compose.yml up -d --build
```

Build the production image only:
```bash
docker build -t rust-api-starter .
```

Run the development stack with hot-reload:
```bash
docker compose -f docker-compose.yml -f compose.override.yml up --build
```

Run only the application container against an existing PostgreSQL instance:
```bash
docker run --rm -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:postgres@host.docker.internal:5432/app \
  -e JWT_SECRET=change-me-to-a-long-random-secret \
  -e JWT_EXPIRES_IN=900 \
  -e JWT_REFRESH_EXPIRES_IN=604800 \
  -e SERVER_PORT=8080 \
  rust-api-starter
```

The production stack exposes `nginx` on port `8080` and proxies requests to the internal `app` service. The image runs the API as a non-root user, creates `/app/logs` and `/app/uploads`, and binds to `0.0.0.0` inside the container.

## Useful Endpoints
- API base: `http://127.0.0.1:8080/api/v1`
- Swagger UI: `http://127.0.0.1:8080/swagger-ui/`
- OpenAPI JSON: `http://127.0.0.1:8080/api-doc/openapi.json`
- Health check: `GET /api/v1/health`
- Upload file: `POST /api/v1/uploads` with multipart field `file` and optional `sub_folder`
- Get static file: `GET /static/{sub_folder}/{file}`

## Development Commands
```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Logging
Logs are written to `LOG_DIR` from `.env` using daily files:
- `logs/INFO_YYYY-MM-DD.log`
- `logs/ERROR_YYYY-MM-DD.log`

Use the CLI helper:
```bash
cargo run --bin logtool -- list
cargo run --bin logtool -- tail --level info --lines 50
cargo run --bin logtool -- pretty --level error --lines 20
```

## Configuration
Main environment variables are defined in [.env.example](/Users/un/Documents/workshop/rust-api-starter/.env.example):
- `DATABASE_URL`
- `JWT_SECRET`
- `JWT_EXPIRES_IN`
- `JWT_REFRESH_EXPIRES_IN`
- `LOG_DIR`
- `UPLOAD_DIR`
- `RATE_LIMIT_REQUESTS`
- `RATE_LIMIT_WINDOW_SECONDS`
- `RUST_LOG`
- `SERVER_HOST`
- `SERVER_PORT`

## Documentation And Knowledge Base
Repository documentation lives in [docs/README.md](/Users/un/Documents/workshop/rust-api-starter/docs/README.md).

Recommended reading order for new engineers:
1. [System Overview](/Users/un/Documents/workshop/rust-api-starter/docs/architecture/system-overview.md)
2. [Data Flow](/Users/un/Documents/workshop/rust-api-starter/docs/architecture/data-flow.md)
3. [Module Map](/Users/un/Documents/workshop/rust-api-starter/docs/architecture/module-map.md)
4. [ADRs](/Users/un/Documents/workshop/rust-api-starter/docs/adr/README.md)
5. [Runbooks](/Users/un/Documents/workshop/rust-api-starter/docs/runbooks/README.md)

Use `docs/` for:
- architecture decisions
- bugs and incident history
- impact and root cause records
- change summaries
- prevention notes and runbooks
