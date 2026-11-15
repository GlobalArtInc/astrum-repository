-- rows deleted by the up migration are not restored; tokens of external accounts go too, because older code would read them as local
DELETE FROM access_token WHERE source <> 'default';

ALTER TABLE access_token DROP COLUMN external_subject;
