

/**
 * Parse a gaId into its `format`, `group`, and `name` segments.
 *
 * A gaId is `format:group:name`, `format:name`, or a bare token. Any colons beyond the
 * group separator stay part of the name, so ids whose name legitimately contains a colon
 * are preserved rather than truncated.
 */
export function parseGaCoordinates(gaId: string): { format: string; group: string; name: string } {
  const parts = gaId.split(':');
  if (parts.length >= 3) {
    return { format: parts[0], group: parts[1], name: parts.slice(2).join(':') };
  }
  if (parts.length === 2) {
    return { format: parts[0], group: '', name: parts[1] };
  }
  return { format: '', group: '', name: gaId };
}

/**
 * Build a Package URL (purl) from gaId and version.
 * Format: pkg:{format}/{pkgName}@{version}
 * Example: pkg:maven/org.apache.commons:commons-lang3@3.14.0
 */
export function buildPurl(gaId: string, version: string): string {
  const parts = gaId.split(':');
  if (parts.length < 2) return '';
  const format = parts[0];
  const pkgName = parts.slice(1).join(':');
  return `pkg:${format}/${pkgName}@${version}`;
}

/**
 * Build Maven repository path from gaId and version.
 * Format: {groupId}/{artifactId}/{version} with dots in groupId replaced by slashes.
 * Example: org/apache/commons/commons-lang3/3.14.0
 */
export function buildMavenPath(gaId: string, version: string): string | null {
  const parts = gaId.split(':');
  if (parts.length < 3) return null;
  const [, groupId, artifactId] = parts;
  return `${groupId.replace(/\./g, '/')}/${artifactId}/${version}`;
}

/**
 * Build npm package path.
 * gaId: npm:name or npm:scope:name
 * Path: @scope/name or name (registry path format)
 */
function buildNpmPath(gaId: string, version: string): string | null {
  const parts = gaId.split(':');
  if (parts.length < 2) return null;
  const format = parts[0]?.toLowerCase();
  if (format !== 'npm') return null;
  const name = parts.slice(1).join(':');
  if (!name) return null;
  return `${name}/${version}`;
}

/**
 * Build PyPI package path.
 * gaId: pypi:name
 * Path: name/version/
 */
function buildPypiPath(gaId: string, version: string): string | null {
  const parts = gaId.split(':');
  if (parts.length < 2) return null;
  const format = parts[0]?.toLowerCase();
  if (format !== 'pypi') return null;
  const name = parts[1];
  if (!name) return null;
  return `${name}/${version}`;
}

/**
 * Build NuGet package path.
 * gaId: nuget:packageId
 * Path: packageId/version/
 */
function buildNugetPath(gaId: string, version: string): string | null {
  const parts = gaId.split(':');
  if (parts.length < 2) return null;
  const format = parts[0]?.toLowerCase();
  if (format !== 'nuget') return null;
  const packageId = parts[1];
  if (!packageId) return null;
  return `${packageId}/${version}`;
}

/**
 * Build component path for formats that support it.
 * Returns the repository/registry path for copying.
 * Supported: maven, maven2, npm, pypi, nuget.
 */
export function buildComponentPath(gaId: string, version: string): string | null {
  const parts = gaId.split(':');
  if (parts.length < 2 || !version) return null;
  const format = parts[0]?.toLowerCase();

  switch (format) {
    case 'maven':
    case 'maven2':
      return buildMavenPath(gaId, version);
    case 'npm':
      return buildNpmPath(gaId, version);
    case 'pypi':
      return buildPypiPath(gaId, version);
    case 'nuget':
      return buildNugetPath(gaId, version);
    default:
      return null;
  }
}

/**
 * Get Maven Central (central.globalart.dev) URL for a Maven component.
 * Returns null for non-Maven formats.
 */
export function getMavenCentralUrl(gaId: string): string | null {
  const parts = gaId.split(':');
  const format = parts[0]?.toLowerCase();
  if (parts.length < 3 || (format !== 'maven' && format !== 'maven2')) return null;
  const groupId = parts[1];
  const artifactId = parts[2];
  return `https://central.globalart.dev/artifact/${encodeURIComponent(groupId)}/${encodeURIComponent(artifactId)}`;
}
