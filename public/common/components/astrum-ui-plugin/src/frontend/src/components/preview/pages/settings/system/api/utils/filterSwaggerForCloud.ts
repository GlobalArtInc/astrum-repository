

/**
 * Path keys as they appear in Astrum Swagger 2.0 {@code paths} (relative to {@code basePath}, e.g. {@code /v1/status}).
 * These APIs are not applicable in Repository Cloud; hide them from the preview API documentation UI.
 */
const SELF_HOSTED_ONLY_PATH_PREFIXES: readonly string[] = [
  '/v1/blobstores',
  '/v1/system/node',
  '/v1/nodes',
  '/v1/ssl',
  '/v1/ldap',
  '/v1/security/saml',
  '/v1/security/anonymous',
  '/v1/system/license',
  '/v1/email',
];

function normalizePathKey(key: string): string {
  const k = key.trim();
  if (k.length === 0) {
    return '';
  }
  return k.startsWith('/') ? k : `/${k}`;
}

export function isSelfHostedOnlySwaggerPath(pathKey: string): boolean {
  const k = normalizePathKey(pathKey);
  return SELF_HOSTED_ONLY_PATH_PREFIXES.some((prefix) => k === prefix || k.startsWith(`${prefix}/`));
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

/**
 * Returns a shallow copy of the spec with self-hosted-only paths removed (for cloud UI).
 */
export function filterSwaggerSpecForCloud(spec: Record<string, unknown>): Record<string, unknown> {
  const paths = spec.paths;
  if (!isRecord(paths)) {
    return spec;
  }
  const nextPaths: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(paths)) {
    if (!isSelfHostedOnlySwaggerPath(key)) {
      nextPaths[key] = value;
    }
  }
  return { ...spec, paths: nextPaths };
}
