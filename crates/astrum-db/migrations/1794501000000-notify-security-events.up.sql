CREATE TABLE policy_blocked_notice (
    component_id bigint PRIMARY KEY REFERENCES component (id) ON DELETE CASCADE,
    noticed_at timestamptz NOT NULL DEFAULT now()
);

CREATE FUNCTION notification_capture_events(
    p_action text,
    p_components bigint[],
    p_details text[]
) RETURNS bigint LANGUAGE sql AS $$
    WITH inserted AS (
        INSERT INTO notification_change
            (channel_id, repository, repository_type, format, namespace, name, version, action, component_id, detail)
        SELECT ch.id, r.name, r.type, c.format, c.namespace, c.name, c.version, p_action, c.id, e.detail::jsonb
        FROM unnest(p_components, p_details) AS e(component_id, detail)
        JOIN component c ON c.id = e.component_id
        JOIN repository r ON r.name = c.repository
        JOIN notification_channel ch ON ch.enabled
            AND p_action = ANY (ch.events)
            AND (cardinality(ch.repositories) = 0 OR r.name = ANY (ch.repositories))
        RETURNING 1
    )
    SELECT count(*) FROM inserted
$$;
