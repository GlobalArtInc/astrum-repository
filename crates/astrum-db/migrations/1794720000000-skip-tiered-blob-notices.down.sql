DROP TRIGGER notification_asset_replaced ON asset;

CREATE TRIGGER notification_asset_replaced AFTER UPDATE OF blob_id ON asset
    FOR EACH ROW WHEN (NEW.component_id IS NOT NULL AND OLD.blob_id IS DISTINCT FROM NEW.blob_id)
    EXECUTE FUNCTION notification_asset_stored();
