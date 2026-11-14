CREATE OR REPLACE FUNCTION notification_component_changed() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        PERFORM notification_capture(NEW.repository, NEW.format, NEW.namespace, NEW.name, NEW.version,
            CASE WHEN nullif(current_setting('astrum.action', true), '') = 'promoted' THEN 'promoted' ELSE 'published' END,
            NEW.id, current_setting('astrum.initiator', true));
        RETURN NEW;
    END IF;
    PERFORM notification_capture(OLD.repository, OLD.format, OLD.namespace, OLD.name, OLD.version,
        'deleted', OLD.id, current_setting('astrum.initiator', true));
    RETURN OLD;
END
$$;

CREATE OR REPLACE FUNCTION notification_asset_stored() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    v_action text := CASE WHEN current_setting('astrum.action', true) = 'promoted' THEN 'promoted' ELSE 'updated' END;
BEGIN
    PERFORM notification_capture(c.repository, c.format, c.namespace, c.name, c.version,
        v_action, c.id, NEW.uploader)
    FROM component c
    WHERE c.id = NEW.component_id;
    RETURN NEW;
END
$$;

UPDATE notification_channel
SET events = array_append(events, 'promoted')
WHERE events @> ARRAY['published', 'updated', 'deleted']
  AND NOT events @> ARRAY['promoted'];
