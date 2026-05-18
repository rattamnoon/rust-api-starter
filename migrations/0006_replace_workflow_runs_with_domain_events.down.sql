DROP INDEX IF EXISTS domain_events_aggregate_idx;
DROP INDEX IF EXISTS domain_events_topic_idx;
DROP INDEX IF EXISTS domain_events_publish_status_idx;
DROP TABLE IF EXISTS domain_events;

CREATE TABLE workflow_runs (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    order_id uuid NOT NULL UNIQUE REFERENCES orders(id) ON DELETE CASCADE,
    workflow_id text NOT NULL,
    namespace text NOT NULL,
    task_queue text NOT NULL,
    status text NOT NULL,
    last_error text NULL,
    started_at timestamptz NULL,
    finished_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
