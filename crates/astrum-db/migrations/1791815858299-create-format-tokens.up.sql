CREATE TABLE format_token (
    token_hash text PRIMARY KEY,
    realm text NOT NULL,
    user_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz,
    last_used_at timestamptz
);

CREATE INDEX format_token_by_user ON format_token (user_id);
