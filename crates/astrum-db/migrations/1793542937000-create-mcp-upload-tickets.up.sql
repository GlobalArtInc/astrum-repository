CREATE TABLE mcp_upload_ticket (
    ticket_hash text PRIMARY KEY,
    user_id text NOT NULL,
    realm text NOT NULL,
    scope text NOT NULL,
    repository text NOT NULL,
    fields jsonb NOT NULL,
    file_field text NOT NULL,
    file_name text NOT NULL,
    max_bytes bigint NOT NULL,
    client text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL
);

CREATE INDEX mcp_upload_ticket_expiry ON mcp_upload_ticket (expires_at);

CREATE INDEX mcp_upload_ticket_by_user ON mcp_upload_ticket (user_id);
