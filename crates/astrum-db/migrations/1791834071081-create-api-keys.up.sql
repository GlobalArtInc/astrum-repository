CREATE TABLE api_key (
    realm text NOT NULL,
    user_id text NOT NULL,
    key_hash text NOT NULL,
    sealed_key text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz,
    PRIMARY KEY (realm, user_id)
);

CREATE UNIQUE INDEX api_key_by_hash ON api_key (realm, key_hash);
CREATE INDEX api_key_by_user ON api_key (user_id);
