CREATE FUNCTION license_mark_component() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    changed bigint;
BEGIN
    IF TG_OP = 'DELETE' THEN
        changed := OLD.component_id;
    ELSE
        changed := NEW.component_id;
    END IF;
    INSERT INTO policy_dirty (repository, subject)
    SELECT c.repository, 'c:' || c.id
    FROM component c
    JOIN repository r ON r.name = c.repository
    WHERE c.id = changed AND r.attributes #>> '{policy,name}' IS NOT NULL
    ON CONFLICT (repository, subject) DO UPDATE SET queued_at = policy_dirty.queued_at;
    RETURN NULL;
END
$$;

CREATE TRIGGER license_inserted AFTER INSERT ON component_license
    FOR EACH ROW EXECUTE FUNCTION license_mark_component();

CREATE TRIGGER license_changed AFTER UPDATE OF state, expression, ids ON component_license
    FOR EACH ROW WHEN (OLD.state IS DISTINCT FROM NEW.state
        OR OLD.expression IS DISTINCT FROM NEW.expression
        OR OLD.ids IS DISTINCT FROM NEW.ids)
    EXECUTE FUNCTION license_mark_component();

CREATE TRIGGER license_removed AFTER DELETE ON component_license
    FOR EACH ROW EXECUTE FUNCTION license_mark_component();

CREATE INDEX policy_verdict_reasons ON policy_verdict USING gin (reasons jsonb_path_ops)
    WHERE reasons <> '[]'::jsonb;

WITH changed AS (
    UPDATE content_policy SET revision = revision + 1, updated_at = now()
    WHERE licenses ->> 'mode' IN ('report', 'block')
    RETURNING 1
)
INSERT INTO revision (name, value)
SELECT 'policies', 1 WHERE EXISTS (SELECT 1 FROM changed)
ON CONFLICT (name) DO UPDATE SET value = revision.value + 1;
