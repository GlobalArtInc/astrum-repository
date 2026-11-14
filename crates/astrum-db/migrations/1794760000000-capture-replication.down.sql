DROP TRIGGER replication_asset_replaced ON asset;

DROP TRIGGER replication_asset_created ON asset;

DROP TRIGGER replication_component_deleted ON component;

DROP TRIGGER replication_component_created ON component;

DROP FUNCTION replication_asset_stored();

DROP FUNCTION replication_component_changed();

DROP FUNCTION replication_capturing(text);

DROP FUNCTION replication_mark(text, text, text, text);

DROP TABLE replication_sent;
