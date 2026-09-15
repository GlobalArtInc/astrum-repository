CREATE TABLE ansible_import (
    id uuid PRIMARY KEY,
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    state text NOT NULL,
    namespace text,
    name text,
    version text,
    messages jsonb NOT NULL DEFAULT '[]',
    error jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz
);

CREATE INDEX ansible_import_created ON ansible_import (created_at);
