ALTER TABLE share_link ADD COLUMN created_source text NOT NULL DEFAULT 'default';

DROP INDEX share_link_by_creator;

CREATE INDEX share_link_by_creator ON share_link (created_source, created_by);
