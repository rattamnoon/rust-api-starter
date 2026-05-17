CREATE TABLE jobs (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    job_type text NOT NULL,
    status text NOT NULL,
    payload jsonb NOT NULL,
    payload_summary text NOT NULL,
    attempt_count integer NOT NULL DEFAULT 0,
    max_attempts integer NOT NULL DEFAULT 3,
    last_error text NULL,
    created_by uuid NULL REFERENCES users(id) ON DELETE SET NULL,
    queued_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz NULL,
    finished_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT jobs_status_valid CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'dead_lettered')),
    CONSTRAINT jobs_attempt_count_nonnegative CHECK (attempt_count >= 0),
    CONSTRAINT jobs_max_attempts_positive CHECK (max_attempts > 0)
);

CREATE TABLE job_attempts (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    job_id uuid NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    attempt_number integer NOT NULL,
    status text NOT NULL,
    error_message text NULL,
    started_at timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT job_attempts_status_valid CHECK (status IN ('running', 'succeeded', 'failed', 'dead_lettered')),
    CONSTRAINT job_attempts_attempt_number_positive CHECK (attempt_number > 0),
    UNIQUE (job_id, attempt_number)
);

CREATE INDEX jobs_status_idx ON jobs (status);
CREATE INDEX jobs_job_type_idx ON jobs (job_type);
CREATE INDEX jobs_created_at_idx ON jobs (created_at DESC);
CREATE INDEX jobs_created_by_idx ON jobs (created_by);
CREATE INDEX job_attempts_job_id_idx ON job_attempts (job_id);
