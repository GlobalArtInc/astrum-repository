CREATE TABLE notification_channel (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    type text NOT NULL,
    enabled boolean NOT NULL DEFAULT true,
    chat_id text,
    message_thread_id bigint,
    recipients text[] NOT NULL DEFAULT '{}',
    secret text,
    target_host text,
    repositories text[] NOT NULL DEFAULT '{}',
    events text[] NOT NULL DEFAULT '{}',
    language text NOT NULL DEFAULT 'ru',
    proxy_mode text NOT NULL DEFAULT 'global',
    proxy_type text,
    proxy_host text,
    proxy_port integer,
    proxy_username text,
    proxy_password text,
    signing_secret text,
    authorization_header text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX notification_channel_name_lower ON notification_channel (lower(name));

CREATE TABLE notification_change (
    id bigserial PRIMARY KEY,
    channel_id uuid NOT NULL REFERENCES notification_channel (id) ON DELETE CASCADE,
    repository text NOT NULL,
    repository_type text NOT NULL,
    format text NOT NULL,
    namespace text NOT NULL,
    name text NOT NULL,
    version text NOT NULL,
    action text NOT NULL,
    component_id bigint,
    initiator text,
    at timestamptz NOT NULL DEFAULT clock_timestamp()
);

CREATE INDEX notification_change_channel ON notification_change (channel_id, id);

CREATE TABLE notification_delivery (
    id bigserial PRIMARY KEY,
    channel_id uuid NOT NULL REFERENCES notification_channel (id) ON DELETE CASCADE,
    delivery_id uuid NOT NULL,
    event text NOT NULL,
    text text NOT NULL,
    notice jsonb,
    written_at timestamptz NOT NULL,
    attempts integer NOT NULL DEFAULT 0,
    next_attempt_at timestamptz NOT NULL DEFAULT now(),
    last_error text
);

CREATE INDEX notification_delivery_due ON notification_delivery (next_attempt_at, id);

CREATE FUNCTION notification_capture(
    p_repository text,
    p_format text,
    p_namespace text,
    p_name text,
    p_version text,
    p_action text,
    p_component bigint,
    p_initiator text
) RETURNS void LANGUAGE sql AS $$
    INSERT INTO notification_change
        (channel_id, repository, repository_type, format, namespace, name, version, action, component_id, initiator)
    SELECT c.id, r.name, r.type, p_format, p_namespace, p_name, p_version, p_action, p_component,
           nullif(p_initiator, '')
    FROM notification_channel c
    JOIN repository r ON r.name = p_repository
    WHERE c.enabled
      AND ((cardinality(c.repositories) = 0 AND r.type = 'hosted') OR r.name = ANY (c.repositories))
$$;

CREATE FUNCTION notification_component_changed() RETURNS trigger LANGUAGE plpgsql AS $$
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

CREATE FUNCTION notification_asset_stored() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM notification_capture(c.repository, c.format, c.namespace, c.name, c.version,
        'updated', c.id, NEW.uploader)
    FROM component c
    WHERE c.id = NEW.component_id;
    RETURN NEW;
END
$$;

CREATE TRIGGER notification_component_created AFTER INSERT ON component
    FOR EACH ROW EXECUTE FUNCTION notification_component_changed();

CREATE TRIGGER notification_component_deleted AFTER DELETE ON component
    FOR EACH ROW EXECUTE FUNCTION notification_component_changed();

CREATE TRIGGER notification_asset_created AFTER INSERT ON asset
    FOR EACH ROW WHEN (NEW.component_id IS NOT NULL)
    EXECUTE FUNCTION notification_asset_stored();

CREATE TRIGGER notification_asset_replaced AFTER UPDATE OF blob_id ON asset
    FOR EACH ROW WHEN (NEW.component_id IS NOT NULL AND OLD.blob_id IS DISTINCT FROM NEW.blob_id)
    EXECUTE FUNCTION notification_asset_stored();
