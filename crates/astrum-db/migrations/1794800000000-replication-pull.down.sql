DELETE FROM task
WHERE type = 'replication.reconcile'
    AND properties ->> 'replicationId' IN (SELECT id::text FROM replication WHERE direction = 'pull');

DELETE FROM replication WHERE direction = 'pull';

CREATE OR REPLACE FUNCTION replication_capturing(p_repository text) RETURNS boolean LANGUAGE sql STABLE AS $$
    SELECT coalesce(current_setting('astrum.action', true), '') NOT IN ('replicated', 'tiered')
        AND EXISTS (SELECT 1 FROM replication WHERE enabled AND source_repository = p_repository)
$$;

CREATE OR REPLACE FUNCTION replication_mark(
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

DROP INDEX replication_source_enabled;

CREATE INDEX replication_source_enabled ON replication (source_repository) WHERE enabled;

ALTER TABLE replication DROP COLUMN direction;
