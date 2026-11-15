CREATE TABLE replication_remote (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    url text NOT NULL,
    token text NOT NULL,
    use_trust_store boolean NOT NULL DEFAULT false,
    allow_private_network boolean NOT NULL DEFAULT false,
    max_concurrency integer NOT NULL DEFAULT 2 CHECK (max_concurrency BETWEEN 1 AND 16),
    max_bytes_per_second bigint CHECK (max_bytes_per_second IS NULL OR max_bytes_per_second > 0),
    remote_instance_id uuid,
    remote_version text,
    checked_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX replication_remote_name_lower ON replication_remote (lower(name));

CREATE TABLE replication (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    source_repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    remote_id uuid NOT NULL REFERENCES replication_remote (id) ON DELETE RESTRICT,
    remote_repository text NOT NULL,
    enabled boolean NOT NULL DEFAULT true,
    name_filter text,
    version_filter text,
    propagate_deletes boolean NOT NULL DEFAULT false,
    settle_seconds integer NOT NULL DEFAULT 30 CHECK (settle_seconds BETWEEN 0 AND 3600),
    reconcile_minutes integer NOT NULL DEFAULT 360 CHECK (reconcile_minutes >= 5),
    alert_channels uuid[] NOT NULL DEFAULT '{}',
    alert_state text NOT NULL DEFAULT 'ok',
    last_success_at timestamptz,
    last_failure_at timestamptz,
    last_error text,
    last_reconcile_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (source_repository, remote_id, remote_repository)
);

CREATE UNIQUE INDEX replication_name_lower ON replication (lower(name));

CREATE INDEX replication_source_enabled ON replication (source_repository) WHERE enabled;

CREATE TABLE replication_item (
    id bigserial PRIMARY KEY,
    replication_id uuid NOT NULL REFERENCES replication (id) ON DELETE CASCADE,
    namespace text NOT NULL,
    name text NOT NULL,
    version text NOT NULL,
    state text NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'leased', 'dead')),
    reason text NOT NULL,
    generation bigint NOT NULL DEFAULT 1,
    first_queued_at timestamptz NOT NULL DEFAULT now(),
    not_before timestamptz NOT NULL DEFAULT now(),
    attempts integer NOT NULL DEFAULT 0,
    leased_until timestamptz,
    last_error text,
    UNIQUE (replication_id, namespace, name, version)
);

CREATE INDEX replication_item_due ON replication_item (replication_id, not_before) WHERE state = 'pending';

CREATE INDEX replication_item_dead ON replication_item (replication_id) WHERE state = 'dead';

CREATE INDEX replication_item_lease ON replication_item (leased_until) WHERE state = 'leased';
