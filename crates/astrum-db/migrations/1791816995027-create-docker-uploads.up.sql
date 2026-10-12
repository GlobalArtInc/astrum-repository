CREATE TABLE docker_upload (
    id uuid PRIMARY KEY,
    repository text NOT NULL,
    image text NOT NULL,
    blob_store text NOT NULL,
    size bigint NOT NULL DEFAULT 0,
    chunks jsonb NOT NULL DEFAULT '[]',
    created_by text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX docker_upload_updated ON docker_upload (updated_at);

CREATE INDEX asset_docker_name ON asset (repository, (attributes #>> '{docker,name}'))
    WHERE format = 'docker';

CREATE INDEX asset_docker_subject ON asset (repository, (attributes #>> '{docker,subject}'))
    WHERE format = 'docker';
