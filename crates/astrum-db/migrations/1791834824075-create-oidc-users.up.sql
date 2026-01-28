CREATE TABLE security_oidc_user (
    user_id text PRIMARY KEY,
    subject text NOT NULL DEFAULT '',
    first_name text NOT NULL DEFAULT '',
    last_name text NOT NULL DEFAULT '',
    email text NOT NULL DEFAULT '',
    groups jsonb NOT NULL DEFAULT '[]',
    last_login_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now()
);
