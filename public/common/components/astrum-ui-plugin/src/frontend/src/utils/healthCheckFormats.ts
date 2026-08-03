

/**
 * Proxy repository formats that support Repository Health Check in the UI (Browse / Protect).
 *
 * Keep in sync with HealthCheckCell. Backend may further exclude some repos (e.g. Maven snapshot policy);
 * details API does not expose that, so we use format-level eligibility here.
 */
const HEALTH_CHECK_SUPPORTED_FORMATS = new Set([
  'maven2',
  'npm',
  'nuget',
  'pypi',
  'rubygems',
  'cocoapods',
  'conan',
  'conda',
  'go',
  'r',
  'apt',
]);

/**
 * Returns true when Health Check can be enabled for this repository format (proxy only — call sites filter type).
 */
export function isHealthCheckSupportedFormat(format: string | null | undefined): boolean {
  return HEALTH_CHECK_SUPPORTED_FORMATS.has(format?.toLowerCase?.() ?? '');
}
