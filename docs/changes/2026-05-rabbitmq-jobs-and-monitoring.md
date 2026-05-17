# Change: RabbitMQ Jobs And Monitoring

## Summary
Added a durable background job system using RabbitMQ, a dedicated worker process, PostgreSQL-backed job tracking, and Prometheus/Grafana monitoring for queue and processing visibility.

## Motivation
The API needed an asynchronous execution path for non-request work such as welcome email dispatch and uploaded-file processing, plus operational charts for queue health and job outcomes.

## Affected Flows
- `POST /api/v1/auth/register` now enqueues a welcome-email job.
- `POST /api/v1/uploads` now enqueues a file-processing job after persisting file metadata.
- Admins can inspect jobs, chart summaries, and retry failed jobs from `/api/v1/jobs/*`.
- The worker consumes messages from RabbitMQ and updates job status history in PostgreSQL.

## Modules/Services Changed
- `jobs` module for job APIs, persistence, charts, and retry flow
- shared RabbitMQ publisher/consumer layer
- worker binary
- Docker stack with `rabbitmq`, `worker`, `prometheus`, and `grafana`

## Backward Compatibility
Existing auth, upload, and user endpoints remain in place. The main behavior change is that register and upload now depend on queue availability to enqueue follow-up work.

## Operational Notes
- `GET /metrics` exists on the app service for Prometheus scraping and is blocked at nginx.
- RabbitMQ management UI is exposed locally on `127.0.0.1:15672`.
- Grafana is exposed locally on `127.0.0.1:3000` with default credentials from compose.

## References
- `migrations/0004_create_jobs.up.sql`
- `src/modules/jobs/`
- `src/shared/queue/`
- `src/bin/worker.rs`
