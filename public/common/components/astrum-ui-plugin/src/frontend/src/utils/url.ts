

/**
 * Ensures a URL ends with a trailing slash.
 * @param url - The URL to process
 * @returns URL with trailing slash
 */
export function ensureTrailingSlash(url: string): string {
  return url.endsWith('/') ? url : `${url}/`;
}
