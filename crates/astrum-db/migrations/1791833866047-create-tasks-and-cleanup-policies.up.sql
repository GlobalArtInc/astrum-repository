CREATE TABLE cleanup_policy (
    name text PRIMARY KEY,
    format text NOT NULL,
    notes text NOT NULL DEFAULT '',
    criteria jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX cleanup_policy_name_lower ON cleanup_policy (lower(name));

CREATE TABLE task (
    id uuid PRIMARY KEY,
    type text NOT NULL,
    name text NOT NULL,
    enabled boolean NOT NULL DEFAULT true,
    frequency jsonb NOT NULL DEFAULT '{"schedule": "manual"}',
    alert_email text,
    notification_condition text NOT NULL DEFAULT 'FAILURE',
    properties jsonb NOT NULL DEFAULT '{}',
    next_run_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX task_name_lower ON task (lower(name));
CREATE INDEX task_due ON task (next_run_at) WHERE enabled AND next_run_at IS NOT NULL;

CREATE TABLE task_run (
    id bigserial PRIMARY KEY,
    task_id uuid NOT NULL REFERENCES task (id) ON DELETE CASCADE,
    state text NOT NULL,
    trigger text NOT NULL,
    initiator text NOT NULL DEFAULT '',
    node text NOT NULL DEFAULT '',
    message text NOT NULL DEFAULT '',
    counts jsonb NOT NULL DEFAULT '{}',
    cancel_requested boolean NOT NULL DEFAULT false,
    queued_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    finished_at timestamptz
);

CREATE INDEX task_run_task ON task_run (task_id, id DESC);
CREATE INDEX task_run_pending ON task_run (state, queued_at) WHERE state IN ('waiting', 'running');
CREATE UNIQUE INDEX task_run_one_active ON task_run (task_id) WHERE state IN ('waiting', 'running');

CREATE TABLE task_log (
    run_id bigint NOT NULL REFERENCES task_run (id) ON DELETE CASCADE,
    line integer NOT NULL,
    at timestamptz NOT NULL DEFAULT now(),
    level text NOT NULL,
    message text NOT NULL,
    PRIMARY KEY (run_id, line)
);
