# Local Debugging

## Goal
Use this runbook to debug the service locally.

## Steps
1. Start PostgreSQL with `docker compose up -d`.
2. Copy `.env.example` to `.env` if needed.
3. Run the API with `cargo run --bin rust-api-starter`.
4. Open Swagger UI at `/swagger-ui/`.
5. Inspect logs with:
   - `cargo run --bin logtool -- list`
   - `cargo run --bin logtool -- tail --level info --lines 50`
   - `cargo run --bin logtool -- pretty --level error --lines 20`

## Common Checks
- verify migrations ran successfully
- check `logs/ERROR_YYYY-MM-DD.log`
- verify rate limiting is not blocking your local test flow

## Related Docs
- [System Overview](../architecture/system-overview.md)
- [Data Flow](../architecture/data-flow.md)
