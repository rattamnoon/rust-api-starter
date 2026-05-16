# Module Map

## HTTP Layer
- `src/main.rs`: builds the server and global middleware stack
- `src/app.rs`: mounts `/api/v1`, Swagger UI, and module routes
- `src/middleware/auth_middleware.rs`: bearer-token auth and current-user injection
- `src/middleware/request_logging.rs`: request completion logging
- `src/middleware/rate_limit.rs`: per-client-IP rate limiting

## Feature Modules
### Auth
- `handler.rs`: HTTP endpoints
- `service.rs`: register/login/refresh/logout/me logic
- `repository.rs`: user and refresh-token persistence
- `model.rs`: refresh-token row model
- `dto.rs`: auth request/response payloads
- `routes.rs`: route registration

### Users
- `handler.rs`: HTTP endpoints
- `service.rs`: CRUD logic and authorization checks
- `repository.rs`: user SQL queries
- `model.rs`: user row model
- `dto.rs`: request/query/response payloads
- `routes.rs`: route registration

## Shared Infrastructure
- `src/config`: environment-driven settings
- `src/db`: database pool bootstrap
- `src/errors`: centralized application errors and HTTP mapping
- `src/logging`: tracing/file log setup and CLI helpers
- `src/shared`: JWT, password hashing, app state, rate limiter, response helpers, extractor types

## Ownership Notes
- Business rules should be added to services first, not handlers.
- Shared reusable logic belongs in `src/shared` or `src/middleware`, not copied into modules.
