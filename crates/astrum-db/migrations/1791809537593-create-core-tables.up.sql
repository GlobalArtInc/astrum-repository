CREATE TABLE security_user (
    id text PRIMARY KEY,
    first_name text NOT NULL DEFAULT '',
    last_name text NOT NULL DEFAULT '',
    email text NOT NULL DEFAULT '',
    password_hash text,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE security_role (
    id text PRIMARY KEY,
    source text NOT NULL DEFAULT 'default',
    name text NOT NULL,
    description text NOT NULL DEFAULT '',
    privileges jsonb NOT NULL DEFAULT '[]',
    roles jsonb NOT NULL DEFAULT '[]',
    read_only boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE security_privilege (
    id text PRIMARY KEY,
    type text NOT NULL,
    description text NOT NULL DEFAULT '',
    properties jsonb NOT NULL DEFAULT '{}',
    read_only boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE security_user_role (
    user_id text NOT NULL,
    source text NOT NULL,
    roles jsonb NOT NULL DEFAULT '[]',
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, source)
);

CREATE TABLE content_selector (
    name text PRIMARY KEY,
    type text NOT NULL DEFAULT 'csel',
    description text NOT NULL DEFAULT '',
    expression text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE blob_store (
    name text PRIMARY KEY,
    type text NOT NULL,
    attributes jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX blob_store_name_ci ON blob_store (lower(name));

CREATE TABLE repository (
    name text PRIMARY KEY,
    format text NOT NULL,
    type text NOT NULL,
    online boolean NOT NULL DEFAULT true,
    blob_store text REFERENCES blob_store (name) ON UPDATE CASCADE ON DELETE RESTRICT,
    attributes jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX repository_name_ci ON repository (lower(name));

CREATE TABLE audit_event (
    id bigserial PRIMARY KEY,
    at timestamptz NOT NULL DEFAULT now(),
    domain text NOT NULL,
    type text NOT NULL,
    context text NOT NULL DEFAULT '',
    initiator text NOT NULL DEFAULT '',
    attributes jsonb NOT NULL DEFAULT '{}'
);

CREATE INDEX audit_event_at ON audit_event (at);
