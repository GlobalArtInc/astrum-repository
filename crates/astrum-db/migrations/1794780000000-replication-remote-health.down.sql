ALTER TABLE replication_remote
    DROP COLUMN last_error,
    DROP COLUMN suspended_until,
    DROP COLUMN failures;
