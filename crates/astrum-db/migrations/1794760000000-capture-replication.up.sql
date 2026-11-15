CREATE TABLE replication_sent (
    replication_id uuid NOT NULL REFERENCES replication (id) ON DELETE CASCADE,
    namespace text NOT NULL,
    name text NOT NULL,
    version text NOT NULL,
    fingerprint text NOT NULL,
    origin text NOT NULL,
    sent_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (replication_id, namespace, name, version)
);

CREATE FUNCTION replication_mark(
    p_repository text,
    p_namespace text,
    p_name text,
    p_version text
) RETURNS void LANGUAGE sql AS $$
    INSERT INTO replication_item (replication_id, namespace, name, version, reason, not_before)
    SELECT r.id, p_namespace, p_name, p_version, 'event', now() + make_interval(secs => r.settle_seconds)
    FROM replication r
    WHERE r.enabled AND r.source_repository = p_repository
    ON CONFLICT (replication_id, namespace, name, version) DO UPDATE SET
        generation = replication_item.generation + 1,
        not_before = EXCLUDED.not_before,
        reason = 'event',
        state = CASE WHEN replication_item.state = 'leased' THEN 'leased' ELSE 'pending' END,
        attempts = CASE WHEN replication_item.state = 'dead' THEN 0 ELSE replication_item.attempts END
$$;

CREATE FUNCTION replication_capturing(p_repository text) RETURNS boolean LANGUAGE sql STABLE AS $$
    SELECT coalesce(current_setting('astrum.action', true), '') NOT IN ('replicated', 'tiered')
        AND EXISTS (SELECT 1 FROM replication WHERE enabled AND source_repository = p_repository)
$$;

CREATE FUNCTION replication_component_changed() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF replication_capturing(NEW.repository) THEN
            PERFORM replication_mark(NEW.repository, NEW.namespace, NEW.name, NEW.version);
        END IF;
        RETURN NEW;
    END IF;
    IF replication_capturing(OLD.repository) THEN
        PERFORM replication_mark(OLD.repository, OLD.namespace, OLD.name, OLD.version);
    END IF;
    RETURN OLD;
END
$$;

CREATE FUNCTION replication_asset_stored() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF replication_capturing(NEW.repository) THEN
        PERFORM replication_mark(c.repository, c.namespace, c.name, c.version)
        FROM component c
        WHERE c.id = NEW.component_id;
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER replication_component_created AFTER INSERT ON component
    FOR EACH ROW EXECUTE FUNCTION replication_component_changed();

CREATE TRIGGER replication_component_deleted AFTER DELETE ON component
    FOR EACH ROW EXECUTE FUNCTION replication_component_changed();

CREATE TRIGGER replication_asset_created AFTER INSERT ON asset
    FOR EACH ROW WHEN (NEW.component_id IS NOT NULL)
    EXECUTE FUNCTION replication_asset_stored();

CREATE TRIGGER replication_asset_replaced AFTER UPDATE OF blob_id ON asset
    FOR EACH ROW WHEN (NEW.component_id IS NOT NULL AND OLD.blob_id IS DISTINCT FROM NEW.blob_id)
    EXECUTE FUNCTION replication_asset_stored();
