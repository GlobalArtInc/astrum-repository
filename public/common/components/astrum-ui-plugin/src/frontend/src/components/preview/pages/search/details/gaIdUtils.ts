

/**
 * Shared parsing for the `gaId` route parameter used across the GA detail hooks.
 *
 * Not to be confused with the `parseGaId` in `core/search.routes.ts`, which URL-decodes an
 * encoded route segment. This one splits an already-decoded identifier into its parts.
 */

export interface ParsedGaId {
  readonly format: string;
  readonly group: string;
  readonly name: string;
}

/**
 * Split a `format:group:name` identifier into its parts.
 *
 * Two- and one-segment inputs are both real: formats without a namespace (npm, raw) produce
 * `format:name`, so the second segment has to be read as the name rather than the group. A
 * single segment cannot be told apart from a bare name, so it is returned as one with no
 * format — callers requiring a format must treat that as unusable rather than query on `''`.
 */
export function parseGaId(gaId: string): ParsedGaId {
  const parts = gaId.split(':');
  if (parts.length >= 3) {
    return { format: parts[0], group: parts[1], name: parts[2] };
  }
  if (parts.length === 2) {
    return { format: parts[0], group: '', name: parts[1] };
  }
  return { format: '', group: '', name: gaId };
}
