CREATE TABLE replication_staged_blob (
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    sha256 text NOT NULL,
    blob_store text NOT NULL,
    blob_id uuid NOT NULL,
    size bigint NOT NULL,
    sha1 text NOT NULL,
    sha512 text NOT NULL,
    md5 text NOT NULL,
    uploaded_by text NOT NULL,
    staged_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL DEFAULT now() + interval '24 hours',
    PRIMARY KEY (repository, sha256)
);

CREATE INDEX replication_staged_blob_expiry ON replication_staged_blob (expires_at);
