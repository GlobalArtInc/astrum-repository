ALTER TABLE replication_remote
    ADD COLUMN failures integer NOT NULL DEFAULT 0,
    ADD COLUMN suspended_until timestamptz,
    ADD COLUMN last_error text;
