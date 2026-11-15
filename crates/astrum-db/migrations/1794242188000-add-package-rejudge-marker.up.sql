ALTER TABLE sca_package ADD COLUMN rejudge_at timestamptz;

CREATE INDEX sca_package_rejudge ON sca_package (id) WHERE rejudge_at IS NOT NULL;
