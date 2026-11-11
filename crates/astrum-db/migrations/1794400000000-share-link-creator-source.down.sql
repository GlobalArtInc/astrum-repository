DROP INDEX share_link_by_creator;

CREATE INDEX share_link_by_creator ON share_link (created_by);

ALTER TABLE share_link DROP COLUMN created_source;
