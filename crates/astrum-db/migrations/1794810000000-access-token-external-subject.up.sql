ALTER TABLE access_token ADD COLUMN external_subject text NULL;

DELETE FROM access_token
WHERE source = 'default'
    AND customer IS NULL
    AND NOT EXISTS (SELECT 1 FROM security_user u WHERE u.id = access_token.user_id);
