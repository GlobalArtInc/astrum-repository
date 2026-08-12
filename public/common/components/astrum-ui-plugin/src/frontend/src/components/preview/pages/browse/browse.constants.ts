

/**
 * Shared constants for the Browse module.
 */

/**
 * Hidden attribute facets that should not be displayed in the Attributes tab.
 * Matches Classic UI's AssetAttributes.js HIDDEN_FACETS.
 */
export const HIDDEN_FACETS = ['npm_rev', 'upstream_globalart_filtered_versions'] as const;

/**
 * Attribute keys that are excluded from dynamic facet rendering.
 * - 'checksum': Rendered separately in a dedicated Checksums section
 * - 'content': Internal backend metadata (timestamps, blob info) not user-facing
 */
export const EXCLUDED_ATTRIBUTE_KEYS = ['checksum', 'content'] as const;

/**
 * Format an attribute value for display.
 * Handles arrays, objects, and primitives.
 *
 * @param value - The value to format
 * @param key - Optional key for context-specific formatting (e.g., 'totalSize' formats as file size)
 * @param formatFileSizeFn - Optional function to format file sizes (injected to avoid circular deps)
 */
export function formatAttributeValue(
  value: unknown,
  key?: string,
  formatFileSizeFn?: (bytes: number) => string
): string {
  if (value === null || value === undefined) return '';
  // Special case: format totalSize as human-readable file size
  if (key === 'totalSize' && typeof value === 'number' && formatFileSizeFn) {
    return formatFileSizeFn(value);
  }
  if (Array.isArray(value)) {
    if (value.length === 0) return '';
    if (typeof value[0] === 'object') {
      return value.map((item) => JSON.stringify(item)).join('\n');
    }
    return value.join(', ');
  }
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}

/**
 * Check if an attribute key should be displayed as a facet section.
 * Filters out hidden facets and excluded keys.
 */
export function shouldDisplayAttributeFacet(
  key: string,
  value: unknown
): boolean {
  return (
    !HIDDEN_FACETS.includes(key as typeof HIDDEN_FACETS[number]) &&
    !EXCLUDED_ATTRIBUTE_KEYS.includes(key as typeof EXCLUDED_ATTRIBUTE_KEYS[number]) &&
    typeof value === 'object' &&
    value !== null
  );
}
