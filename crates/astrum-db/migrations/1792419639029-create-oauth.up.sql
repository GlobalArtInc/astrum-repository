CREATE TABLE oauth_client (
    id text PRIMARY KEY,
    name text NOT NULL,
    redirect_uris jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE oauth_authorization (
    id text PRIMARY KEY,
    client_id text NOT NULL REFERENCES oauth_client (id) ON DELETE CASCADE,
    redirect_uri text NOT NULL,
    code_challenge text NOT NULL,
    requested_scope text NOT NULL,
    state text,
    resource text,
    user_id text,
    realm text,
    granted_scope text,
    code_hash text UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL
);

CREATE INDEX oauth_authorization_expiry ON oauth_authorization (expires_at);

CREATE TABLE oauth_grant (
    id uuid PRIMARY KEY,
    client_id text NOT NULL REFERENCES oauth_client (id) ON DELETE CASCADE,
    user_id text NOT NULL,
    realm text NOT NULL,
    scope text NOT NULL,
    resource text,
    access_hash text NOT NULL UNIQUE,
    access_expires_at timestamptz NOT NULL,
    refresh_hash text NOT NULL UNIQUE,
    refresh_expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz
);

CREATE INDEX oauth_grant_by_user ON oauth_grant (user_id);
