# System Overview

## Purpose
This service is a Rust backend starter for web APIs using Actix Web, SQLx, PostgreSQL, JWT authentication, Swagger docs, request logging, and rate limiting.

## High-Level Components
- `src/main.rs`: process bootstrap, config loading, tracing/logging init, DB migration, HTTP server startup
- `src/app.rs`: route registration and shared HTTP wiring
- `src/modules/auth`: register, login, refresh, logout, current-user profile
- `src/modules/users`: authenticated user CRUD with role-based authorization
- `src/shared`: cross-cutting helpers such as JWT, password hashing, app state, rate limiting types
- `src/middleware`: auth, request logging, and rate limiting middleware
- `migrations/`: schema history for PostgreSQL

## Core Runtime Dependencies
- Actix Web for HTTP handling
- SQLx for PostgreSQL access
- PostgreSQL 18 for persistence
- JWT for access and refresh tokens
- Argon2 for password hashing

## Architectural Rules
- Handlers should only parse input, call services, and return responses.
- Services own business logic and authorization rules.
- Repositories contain SQL/database access only.
- Models represent database rows.
- DTOs represent request and response payloads.

## Related Docs
- [Module Map](./module-map.md)
- [Data Flow](./data-flow.md)
- [External Dependencies](./external-dependencies.md)
