DELETE FROM access_token WHERE customer IS NOT NULL OR repositories IS NOT NULL;

DROP TABLE token_download_day;

DROP INDEX access_token_by_customer;

ALTER TABLE access_token
    DROP CONSTRAINT access_token_customer_shape,
    DROP CONSTRAINT access_token_narrowed_reads,
    DROP COLUMN expiry_notice,
    DROP COLUMN last_used_ip,
    DROP COLUMN created_by,
    DROP COLUMN repositories,
    DROP COLUMN customer;

DROP TABLE customer;
