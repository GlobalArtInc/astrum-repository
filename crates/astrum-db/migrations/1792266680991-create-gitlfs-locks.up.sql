CREATE TABLE gitlfs_lock (
    id bigserial PRIMARY KEY,
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    path text NOT NULL,
    ref_name text,
    owner text NOT NULL,
    locked_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (repository, path)
);
