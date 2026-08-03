

/**
 * Whether GlobalArt-internal developer tooling should be enabled.
 * Currently gates the Test Hub, but is intended to cover other internal-only
 * developer tooling going forward (as opposed to `?debug`, which stays scoped
 * to loading changed files from the filesystem).
 * True when the GLOBALART_INTERNAL build flag was set at build time, or when a developer
 * has opted in at runtime via `localStorage.setItem('GLOBALART_INTERNAL', 'true')`.
 */
export function isGlobalArtDevMode() {
  if (typeof __GLOBALART_INTERNAL__ !== 'undefined' && __GLOBALART_INTERNAL__) {
    return true;
  }
  try {
    return typeof localStorage !== 'undefined' && localStorage.getItem('GLOBALART_INTERNAL') === 'true';
  } catch {
    return false;
  }
}
