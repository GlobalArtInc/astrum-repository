UPDATE notification_channel SET events = array_remove(events, 'promoted');

CREATE OR REPLACE FUNCTION notification_component_changed() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        PERFORM notification_capture(NEW.repository, NEW.format, NEW.namespace, NEW.name, NEW.version,
            'published', NEW.id, current_setting('astrum.initiator', true));
        RETURN NEW;
    END IF;
    PERFORM notification_capture(OLD.repository, OLD.format, OLD.namespace, OLD.name, OLD.version,
        'deleted', OLD.id, current_setting('astrum.initiator', true));
    RETURN OLD;
END
$$;

CREATE OR REPLACE FUNCTION notification_asset_stored() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM notification_capture(c.repository, c.format, c.namespace, c.name, c.version,
        'updated', c.id, NEW.uploader)
    FROM component c
    WHERE c.id = NEW.component_id;
    RETURN NEW;
END
$$;
