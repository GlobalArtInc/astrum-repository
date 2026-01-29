DELETE FROM access_token WHERE expires_at IS NULL;
ALTER TABLE access_token ALTER COLUMN expires_at SET NOT NULL;
