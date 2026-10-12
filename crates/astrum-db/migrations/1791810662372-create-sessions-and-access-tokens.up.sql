CREATE TABLE user_session (
    token_hash text PRIMARY KEY,
    user_id text NOT NULL,
    realm text NOT NULL,
    credential_stamp text NOT NULL,
    csrf_token text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    client_ip text NOT NULL DEFAULT '',
    user_agent text NOT NULL DEFAULT ''
);

CREATE INDEX user_session_by_user ON user_session (user_id);

CREATE TABLE access_token (
    id uuid PRIMARY KEY,
    user_id text NOT NULL,
    source text NOT NULL DEFAULT 'default',
    name text NOT NULL,
    token_hash text NOT NULL UNIQUE,
    token_hint text NOT NULL,
    scope text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    last_used_at timestamptz
);

CREATE INDEX access_token_by_user ON access_token (user_id);
