CREATE TABLE repository_usage_day (
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    day date NOT NULL,
    components bigint NOT NULL,
    assets bigint NOT NULL,
    bytes bigint NOT NULL,
    downloads bigint NOT NULL,
    PRIMARY KEY (repository, day)
);

CREATE INDEX repository_usage_day_day ON repository_usage_day (day);

CREATE TABLE blob_store_usage_day (
    store text NOT NULL REFERENCES blob_store (name) ON UPDATE CASCADE ON DELETE CASCADE,
    day date NOT NULL,
    blob_count bigint NOT NULL,
    total_bytes bigint NOT NULL,
    available_bytes bigint,
    PRIMARY KEY (store, day)
);
