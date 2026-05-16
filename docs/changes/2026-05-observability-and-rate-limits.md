# Change: Observability And Rate Limits

## Summary
Added structured file logging, a CLI tool for reading logs, request logging, and global in-memory rate limiting.

## Motivation
The service needed better local observability and basic request protection during development and early deployment.

## Affected Flows
- request lifecycle
- operational debugging
- rate-limited HTTP responses

## Modules/Services Changed
- `src/logging`
- `src/middleware/request_logging.rs`
- `src/middleware/rate_limit.rs`
- `src/bin/logtool.rs`

## Backward Compatibility
- introduces `429 Too Many Requests` under load
- adds new env vars for log directory and rate-limit settings

## Operational Notes
- logs are written to `./logs`
- rate limiting is process-local and not shared across instances

## References
- [Data Flow](../architecture/data-flow.md)
- [External Dependencies](../architecture/external-dependencies.md)
