# Data Flow

## Request Lifecycle
1. The server accepts an HTTP request in Actix Web.
2. Global middleware applies:
   - rate limiting
   - request logging
   - route-level auth middleware where required
3. The handler validates/parses input DTOs.
4. The handler calls a service method.
5. The service applies business rules and authorization checks.
6. The service calls a repository for persistence or lookup.
7. The repository issues SQLx queries to PostgreSQL.
8. The service maps models to response DTOs.
9. The handler returns the HTTP response.

## Authentication Flow
1. `POST /auth/register` creates a user and returns access/refresh tokens.
2. `POST /auth/login` verifies credentials and rotates refresh tokens.
3. `POST /auth/refresh` validates a refresh token, revokes the old token, and issues a new pair.
4. `GET /auth/me` uses the access token to fetch the current user profile.

## User CRUD Flow
1. Authenticated request reaches `/users` routes.
2. Auth middleware extracts JWT claims into the current-user context.
3. Service authorizes based on role and target user ID.
4. Repository reads/writes the `users` table.

## Failure Flow
- Validation errors are returned by DTO validation.
- Auth and permission failures are returned as `401` or `403`.
- Missing records become `404`.
- Rate-limited requests become `429`.
- Database and internal failures are mapped by `AppError`.
