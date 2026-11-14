CREATE TABLE customer (
    id text PRIMARY KEY CHECK (id ~ '^[a-z0-9-]{1,64}$'),
    name text NOT NULL,
    reference text NOT NULL DEFAULT '',
    note text NOT NULL DEFAULT '',
    disabled boolean NOT NULL DEFAULT false,
    created_by text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE access_token
    ADD COLUMN customer text REFERENCES customer (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN repositories text[],
    ADD COLUMN created_by text,
    ADD COLUMN last_used_ip text,
    ADD COLUMN expiry_notice smallint,
    ADD CONSTRAINT access_token_narrowed_reads CHECK (
        repositories IS NULL OR (scope = 'READ' AND cardinality(repositories) > 0)
    ),
    ADD CONSTRAINT access_token_customer_shape CHECK (
        customer IS NULL OR (
            scope = 'READ'
            AND repositories IS NOT NULL
            AND cardinality(repositories) > 0
            AND expires_at IS NOT NULL
        )
    );

CREATE INDEX access_token_by_customer ON access_token (customer) WHERE customer IS NOT NULL;

CREATE TABLE token_download_day (
    token_id uuid NOT NULL,
    customer text REFERENCES customer (id) ON UPDATE CASCADE ON DELETE CASCADE,
    repository text NOT NULL,
    day date NOT NULL,
    downloads bigint NOT NULL,
    bytes bigint NOT NULL,
    PRIMARY KEY (token_id, repository, day)
);

CREATE INDEX token_download_day_customer ON token_download_day (customer, day) WHERE customer IS NOT NULL;

CREATE INDEX token_download_day_day ON token_download_day (day);
