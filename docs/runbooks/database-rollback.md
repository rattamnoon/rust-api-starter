# Database Rollback

## Goal
Use this runbook when a migration or schema change needs to be backed out safely.

## Steps
1. Identify the migration and affected tables.
2. Confirm whether data loss is acceptable.
3. Review the corresponding `*.down.sql` migration.
4. Communicate impact before running rollback steps in shared environments.
5. After rollback, verify application startup and critical endpoints.

## Risks
- destructive `down.sql` operations can remove schema and data
- application code may no longer match the rolled-back schema

## Follow-up
- document the rollback in `docs/incidents/` or `docs/changes/` if the event is important enough to teach future engineers something
