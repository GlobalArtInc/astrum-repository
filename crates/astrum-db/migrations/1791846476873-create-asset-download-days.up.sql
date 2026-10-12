CREATE TABLE asset_download_day (
    asset_id bigint NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    day date NOT NULL,
    downloads bigint NOT NULL,
    PRIMARY KEY (asset_id, day)
);

CREATE INDEX asset_download_day_day ON asset_download_day (day);
