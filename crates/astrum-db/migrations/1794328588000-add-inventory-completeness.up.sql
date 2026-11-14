ALTER TABLE sca_inventory ADD COLUMN complete boolean NOT NULL DEFAULT true;

CREATE INDEX asset_docker_digest ON asset (repository, (attributes #>> '{docker,digest}'))
    WHERE format = 'docker' AND kind IN ('manifest', 'tag');
