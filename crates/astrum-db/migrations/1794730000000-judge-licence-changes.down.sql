DROP INDEX policy_verdict_reasons;
DROP TRIGGER license_removed ON component_license;
DROP TRIGGER license_changed ON component_license;
DROP TRIGGER license_inserted ON component_license;
DROP FUNCTION license_mark_component();
