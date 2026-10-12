CREATE TABLE blob_store_metrics (
    store text PRIMARY KEY REFERENCES blob_store (name) ON UPDATE CASCADE ON DELETE CASCADE,
    blob_count bigint NOT NULL DEFAULT 0,
    total_bytes bigint NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);
