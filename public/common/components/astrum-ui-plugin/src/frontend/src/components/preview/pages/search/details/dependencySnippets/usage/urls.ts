/**
 * URL helpers shared by the usage generators.
 */

/**
 * The URL with exactly one trailing slash.
 */
export function withSlash(url: string): string {
  return `${url.replace(/\/+$/, '')}/`;
}

/**
 * The URL without trailing slashes.
 */
export function withoutSlash(url: string): string {
  return url.replace(/\/+$/, '');
}

function parse(url: string): URL | null {
  try {
    return new URL(url);
  } catch {
    return null;
  }
}

/**
 * Host and port of the URL (`host:8081`), or the URL itself when it cannot be parsed.
 */
export function hostOf(url: string): string {
  return parse(url)?.host ?? url;
}

/**
 * Host name of the URL without the port.
 */
export function hostnameOf(url: string): string {
  return parse(url)?.hostname ?? url;
}

/**
 * Whether the URL uses plain HTTP, which some clients only accept when told to trust the host.
 */
export function isPlainHttp(url: string): boolean {
  return parse(url)?.protocol === 'http:';
}

/**
 * The asset path as it appears in a URL: no leading slash, every segment percent-encoded once.
 */
export function encodePath(path: string): string {
  return path
    .replace(/^\/+/, '')
    .split('/')
    .map((segment) => {
      let decoded = segment;
      try {
        decoded = decodeURIComponent(segment);
      } catch {
        decoded = segment;
      }
      return encodeURIComponent(decoded);
    })
    .join('/');
}

/**
 * The last segment of a path.
 */
export function fileNameOf(path: string): string {
  const trimmed = path.replace(/\/+$/, '');
  return trimmed.substring(trimmed.lastIndexOf('/') + 1);
}

/**
 * Whether the repository accepts uploads.
 */
export function isHosted(repositoryType: string | null | undefined): boolean {
  return repositoryType === 'hosted';
}
