DROP TRIGGER IF EXISTS repository_usage_asset_delete ON asset;
DROP TRIGGER IF EXISTS repository_usage_asset_resize ON asset;
DROP TRIGGER IF EXISTS repository_usage_asset_insert ON asset;
DROP TRIGGER IF EXISTS repository_usage_repository_insert ON repository;
DROP FUNCTION IF EXISTS repository_usage_assets_removed();
DROP FUNCTION IF EXISTS repository_usage_asset_resized();
DROP FUNCTION IF EXISTS repository_usage_asset_added();
DROP FUNCTION IF EXISTS repository_usage_created();
DROP TABLE repository_usage;
