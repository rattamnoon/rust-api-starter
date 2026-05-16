# ADR-001: Modular Backend Structure

## Status
Accepted

## Context
The team needs a backend structure that is easy to onboard into, keeps business logic out of HTTP handlers, and scales as new modules are added.

## Decision
Use a layered structure with:
- handlers for HTTP only
- services for business logic
- repositories for SQL only
- models for database rows
- DTOs for API payloads
- shared infrastructure for cross-cutting concerns

## Alternatives Considered
- fat handlers with direct database calls
- a flatter structure with fewer folders but weaker boundaries

## Consequences
- More files per feature, but better separation of concerns
- Easier debugging, onboarding, and test placement
- Clearer ownership when bugs span transport, business logic, and persistence

## Related Changes
- [System Overview](../architecture/system-overview.md)
- [Module Map](../architecture/module-map.md)
