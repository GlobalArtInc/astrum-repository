CREATE TABLE repository_usage (
    repository text PRIMARY KEY REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    assets bigint NOT NULL DEFAULT 0,
    bytes bigint NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE FUNCTION repository_usage_created() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO repository_usage (repository) VALUES (NEW.name) ON CONFLICT DO NOTHING;
    RETURN NULL;
END
$$;

CREATE FUNCTION repository_usage_asset_added() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    UPDATE repository_usage
    SET assets = assets + 1, bytes = bytes + NEW.size, updated_at = now()
    WHERE repository = NEW.repository;
    RETURN NULL;
END
$$;

CREATE FUNCTION repository_usage_asset_resized() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.repository = NEW.repository THEN
        UPDATE repository_usage
        SET bytes = bytes + NEW.size - OLD.size, updated_at = now()
        WHERE repository = NEW.repository;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION repository_usage_assets_removed() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM 1 FROM repository_usage
    WHERE repository IN (SELECT DISTINCT repository FROM gone)
    ORDER BY repository
    FOR UPDATE;
    UPDATE repository_usage u
    SET assets = u.assets - d.n, bytes = u.bytes - d.b, updated_at = now()
    FROM (
        SELECT repository, count(*) AS n, coalesce(sum(size), 0)::bigint AS b
        FROM gone
        GROUP BY repository
    ) d
    WHERE u.repository = d.repository;
    RETURN NULL;
END
$$;

CREATE TRIGGER repository_usage_repository_insert AFTER INSERT ON repository
    FOR EACH ROW EXECUTE FUNCTION repository_usage_created();

CREATE TRIGGER repository_usage_asset_insert AFTER INSERT ON asset
    FOR EACH ROW EXECUTE FUNCTION repository_usage_asset_added();

CREATE TRIGGER repository_usage_asset_resize AFTER UPDATE OF size ON asset
    FOR EACH ROW WHEN (OLD.size IS DISTINCT FROM NEW.size)
    EXECUTE FUNCTION repository_usage_asset_resized();

CREATE TRIGGER repository_usage_asset_delete AFTER DELETE ON asset
    REFERENCING OLD TABLE AS gone
    FOR EACH STATEMENT EXECUTE FUNCTION repository_usage_assets_removed();

LOCK TABLE asset IN SHARE MODE;

INSERT INTO repository_usage (repository, assets, bytes)
SELECT r.name, count(a.id), coalesce(sum(a.size), 0)::bigint
FROM repository r
LEFT JOIN asset a ON a.repository = r.name
GROUP BY r.name;
