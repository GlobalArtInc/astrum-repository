CREATE TABLE node (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    version text NOT NULL,
    started_at timestamptz NOT NULL,
    last_seen timestamptz NOT NULL
);
