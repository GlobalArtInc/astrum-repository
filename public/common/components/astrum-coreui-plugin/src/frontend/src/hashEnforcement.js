
/**
 * Extracts the astrum-context-path from the Astrum bundle script URL synchronously.
 * The bundle is always served at {contextPath}/static/rapture/resources/...
 * so we can find any script tag with that path pattern and extract the prefix.
 *
 * Returns '' for a root context path or when the script URL cannot be determined.
 */
export function getContextPath() {
  try {
    const scripts = document.currentScript
      ? [document.currentScript, ...document.scripts]
      : document.scripts;
    for (const script of scripts) {
      if (script.src && script.src.includes('/static/rapture/')) {
        return new URL(script.src).pathname.split('/static/rapture/')[0];
      }
    }
  } catch (e) {
    // ignore - contextPath stays ''
  }
  return '';
}

/**
 * Ensures hash-based routing is active. If someone navigates to a non-hash URL
 * (e.g. /astrum/admin/system/tasks), rewrites to the equivalent hash URL
 * (e.g. /astrum/#/admin/system/tasks) so ui-router can handle routing correctly.
 *
 * Accounts for non-root context paths by detecting the context-path from the
 * Astrum bundle script URL and preserving it in the redirected URL.
 */
export function enforceHashRouting() {
  if (typeof window === 'undefined' || window.__asHashEnforced) {
    return;
  }
  window.__asHashEnforced = true;

  const { hash, pathname, search, origin } = window.location;
  if (!hash && pathname && pathname !== '/') {
    const contextPath = getContextPath();
    const route = contextPath && pathname.startsWith(contextPath)
      ? pathname.slice(contextPath.length)
      : pathname;
    const cleaned = route.replace(/^\/+/, '');
    if (cleaned) {
      const newUrl = `${origin}${contextPath}/${search || ''}#/${cleaned}`;
      window.location.replace(newUrl);
    }
  }
}
