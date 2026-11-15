CREATE TABLE share_link (
    id uuid PRIMARY KEY,
    secret_hash text NOT NULL UNIQUE,
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    path text NOT NULL,
    prefix boolean NOT NULL,
    note text NOT NULL DEFAULT '',
    created_by text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    max_downloads integer CHECK (max_downloads > 0),
    downloads integer NOT NULL DEFAULT 0,
    bytes_served bigint NOT NULL DEFAULT 0,
    last_used_at timestamptz,
    last_used_ip text,
    CHECK (NOT prefix OR max_downloads IS NULL)
);

CREATE INDEX share_link_by_creator ON share_link (created_by);

CREATE INDEX share_link_by_expiry ON share_link (expires_at);
