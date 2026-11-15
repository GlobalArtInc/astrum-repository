CREATE INDEX policy_verdict_oci ON policy_verdict (repository, subject) WHERE subject LIKE 'oci:%';

CREATE FUNCTION component_scan_image_deleted() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO policy_dirty (repository, subject)
    SELECT OLD.repository, 'oci:' || OLD.digest
    WHERE EXISTS (SELECT 1 FROM repository r WHERE r.name = OLD.repository)
    ON CONFLICT (repository, subject) DO NOTHING;
    RETURN NULL;
END
$$;

CREATE TRIGGER component_scan_image_deleted AFTER DELETE ON component_scan
    FOR EACH ROW WHEN (OLD.digest IS NOT NULL)
    EXECUTE FUNCTION component_scan_image_deleted();
