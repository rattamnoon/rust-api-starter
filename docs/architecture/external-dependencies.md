# External Dependencies

## Database
- PostgreSQL 18
- Schema managed through SQLx migrations in `migrations/`

## Security
- JWT for access and refresh tokens
- Argon2 for password hashing

## API Tooling
- Swagger/OpenAPI via `utoipa` and `utoipa-swagger-ui`

## Observability
- `tracing` and `tracing-subscriber`
- file logs in `./logs`
- `logtool` CLI for listing, tailing, grepping, and pretty-printing logs

## Operational Assumptions
- environment variables control config
- Docker Compose provides a local PostgreSQL instance
- rate limiting is currently in-memory and process-local
