CREATE FUNCTION replication_component_revised() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF replication_capturing(NEW.repository) THEN
        PERFORM replication_mark(NEW.repository, NEW.namespace, NEW.name, NEW.version);
    END IF;
    RETURN NEW;
END
$$;

CREATE FUNCTION replication_asset_removed() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF replication_capturing(OLD.repository) THEN
        PERFORM replication_mark(c.repository, c.namespace, c.name, c.version)
        FROM component c
        WHERE c.id = OLD.component_id;
    END IF;
    RETURN OLD;
END
$$;

CREATE FUNCTION replication_asset_moved() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF replication_capturing(NEW.repository) THEN
        PERFORM replication_mark(c.repository, c.namespace, c.name, c.version)
        FROM component c
        WHERE c.id IN (OLD.component_id, NEW.component_id);
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER replication_component_revised AFTER UPDATE OF attributes ON component
    FOR EACH ROW WHEN (OLD.attributes IS DISTINCT FROM NEW.attributes)
    EXECUTE FUNCTION replication_component_revised();

CREATE TRIGGER replication_asset_removed AFTER DELETE ON asset
    FOR EACH ROW WHEN (OLD.component_id IS NOT NULL)
    EXECUTE FUNCTION replication_asset_removed();

CREATE TRIGGER replication_asset_moved AFTER UPDATE OF component_id ON asset
    FOR EACH ROW WHEN (OLD.component_id IS DISTINCT FROM NEW.component_id)
    EXECUTE FUNCTION replication_asset_moved();
