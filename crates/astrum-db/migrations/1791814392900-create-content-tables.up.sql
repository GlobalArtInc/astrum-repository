CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE component (
    id bigserial PRIMARY KEY,
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    format text NOT NULL,
    namespace text NOT NULL DEFAULT '',
    name text NOT NULL,
    version text NOT NULL DEFAULT '',
    kind text NOT NULL DEFAULT '',
    attributes jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (repository, namespace, name, version)
);

CREATE INDEX component_name_trgm ON component USING gin (name gin_trgm_ops);
CREATE INDEX component_namespace_trgm ON component USING gin (namespace gin_trgm_ops);
CREATE INDEX component_format ON component (format);

CREATE TABLE asset (
    id bigserial PRIMARY KEY,
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    format text NOT NULL,
    path text NOT NULL,
    kind text NOT NULL DEFAULT '',
    component_id bigint REFERENCES component (id) ON DELETE CASCADE,
    blob_store text NOT NULL,
    blob_id uuid NOT NULL,
    size bigint NOT NULL,
    content_type text NOT NULL,
    sha1 text NOT NULL,
    sha256 text NOT NULL,
    sha512 text NOT NULL,
    md5 text NOT NULL,
    attributes jsonb NOT NULL DEFAULT '{}',
    uploader text NOT NULL DEFAULT '',
    uploader_ip text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    blob_updated_at timestamptz NOT NULL DEFAULT now(),
    last_downloaded_at timestamptz,
    download_count bigint NOT NULL DEFAULT 0,
    last_verified_at timestamptz,
    remote_etag text,
    remote_last_modified timestamptz,
    UNIQUE (repository, path)
);

CREATE INDEX asset_path_prefix ON asset (repository, path text_pattern_ops);
CREATE INDEX asset_component ON asset (component_id);
CREATE INDEX asset_path_trgm ON asset USING gin (path gin_trgm_ops);
CREATE INDEX asset_sha1 ON asset (sha1);
CREATE INDEX asset_sha256 ON asset (sha256);
CREATE INDEX asset_md5 ON asset (md5);
CREATE INDEX asset_sha512 ON asset (sha512);
CREATE INDEX asset_blob ON asset (blob_store, blob_id);
