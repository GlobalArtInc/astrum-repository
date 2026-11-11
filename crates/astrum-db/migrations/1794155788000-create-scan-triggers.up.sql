CREATE FUNCTION scan_enqueue_component() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF current_setting('astrum.action', true) = 'tiered' THEN
        RETURN NULL;
    END IF;
    INSERT INTO scan_queue (component_id, reason)
    SELECT NEW.id, 'component'
    FROM repository r
    WHERE r.name = NEW.repository AND r.attributes #>> '{policy,name}' IS NOT NULL
    ON CONFLICT (component_id) DO UPDATE SET queued_at = now() WHERE scan_queue.queued_at <> now();
    RETURN NULL;
END
$$;

CREATE FUNCTION scan_enqueue_asset() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF current_setting('astrum.action', true) = 'tiered' THEN
        RETURN NULL;
    END IF;
    INSERT INTO scan_queue (component_id, reason)
    SELECT c.id, 'blob'
    FROM component c
    JOIN repository r ON r.name = c.repository
    WHERE c.id = NEW.component_id AND r.attributes #>> '{policy,name}' IS NOT NULL
    ON CONFLICT (component_id) DO UPDATE SET queued_at = now() WHERE scan_queue.queued_at <> now();
    RETURN NULL;
END
$$;

CREATE TRIGGER scan_component_created AFTER INSERT ON component
    FOR EACH ROW EXECUTE FUNCTION scan_enqueue_component();

CREATE TRIGGER scan_asset_blob_changed AFTER UPDATE OF blob_id, blob_store ON asset
    FOR EACH ROW WHEN (NEW.component_id IS NOT NULL
        AND (OLD.blob_id IS DISTINCT FROM NEW.blob_id OR OLD.blob_store IS DISTINCT FROM NEW.blob_store))
    EXECUTE FUNCTION scan_enqueue_asset();
