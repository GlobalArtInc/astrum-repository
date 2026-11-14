CREATE TABLE blob_hold (
    blob_store text NOT NULL,
    blob_id uuid NOT NULL,
    reason text NOT NULL,
    held_until timestamptz NOT NULL,
    PRIMARY KEY (blob_store, blob_id)
);

CREATE INDEX blob_hold_expiry ON blob_hold (held_until);
