DROP TABLE IF EXISTS workflow_runs;

CREATE TABLE domain_events (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    topic text NOT NULL,
    aggregate_type text NOT NULL,
    aggregate_id uuid NOT NULL,
    event_type text NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    publish_status text NOT NULL DEFAULT 'pending',
    published_at timestamptz NULL,
    last_error text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT domain_events_status_valid CHECK (publish_status IN ('pending', 'published', 'failed'))
);

CREATE INDEX domain_events_publish_status_idx ON domain_events (publish_status, created_at);
CREATE INDEX domain_events_topic_idx ON domain_events (topic, created_at DESC);
CREATE INDEX domain_events_aggregate_idx ON domain_events (aggregate_type, aggregate_id);
