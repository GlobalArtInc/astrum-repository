
/**
 * REST Bootstrap - Phase 2: React-First Shell
 *
 * Fetches auth, permissions, and application state via REST endpoints
 * and seeds window.AS globals BEFORE React renders. This allows
 * ExtJS.useUser(), ExtJS.checkPermission(), ExtJS.state().getValue()
 * to return real data immediately without waiting for ExtJS to boot.
 *
 * When ExtJS eventually loads (~3-5 seconds later), it overwrites
 * window.AS.State and window.AS.Permissions with its own implementations.
 * The React hooks auto-update via ExtJS store event subscriptions.
 *
 * Endpoints used:
 *   GET  /service/extdirect/poll/rapture_State_get  - User, edition, version, state values
 *   POST /service/extdirect                          - rapture_Security.getPermissions
 */

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const STATE_POLL_URL = '/service/extdirect/poll/rapture_State_get';
const EXTDIRECT_URL = '/service/extdirect';
const CSRF_COOKIE_NAME = 'AS-ANTI-CSRF-TOKEN';

// ---------------------------------------------------------------------------
// CSRF Token (same logic as ExtJS Application.js)
// ---------------------------------------------------------------------------

function getCookie(name) {
  if (typeof document === 'undefined') return null;
  const match = document.cookie.match(
    new RegExp('(?:^|;\\s*)' + name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + '=([^;]*)')
  );
  return match ? decodeURIComponent(match[1]) : null;
}

function setCookie(name, value) {
  if (typeof document !== 'undefined') {
    document.cookie = `${name}=${encodeURIComponent(value)}; path=/`;
  }
}

function ensureCSRFToken() {
  let token = getCookie(CSRF_COOKIE_NAME);
  if (!token) {
    token = Math.random().toString();
    setCookie(CSRF_COOKIE_NAME, token);
  }
  return token;
}

// ---------------------------------------------------------------------------
// Shiro WildcardPermission Matching
// Ported from AS.Permissions.implies() in astrum-rapture
// ---------------------------------------------------------------------------

function implies(granted, expected) {
  const grantedParts = granted.split(':');
  const expectedParts = expected.split(':');

  for (let i = 0; i < expectedParts.length; i++) {
    if (i >= grantedParts.length) return true;

    const gSubParts = grantedParts[i].split(',');
    const eSubParts = expectedParts[i].split(',');

    if (gSubParts.includes('*')) continue;

    if (!eSubParts.every((sub) => gSubParts.includes(sub))) {
      return false;
    }
  }

  for (let i = expectedParts.length; i < grantedParts.length; i++) {
    if (!grantedParts[i].split(',').includes('*')) {
      return false;
    }
  }

  return true;
}

// ---------------------------------------------------------------------------
// Permission Check with Cache
// ---------------------------------------------------------------------------

let _permCache = new Map();
let _permSet = null;

function checkPermission(permissions, permission) {
  if (permissions !== _permSet) {
    _permCache = new Map();
    _permSet = permissions;
  }

  const lower = permission.toLowerCase();
  const cached = _permCache.get(lower);
  if (cached !== undefined) return cached;

  let result = false;
  if (permissions.has(lower)) {
    result = true;
  } else {
    for (const granted of permissions) {
      if (implies(granted, lower)) {
        result = true;
        break;
      }
    }
  }

  _permCache.set(lower, result);
  return result;
}

// ---------------------------------------------------------------------------
// REST Fetchers
// ---------------------------------------------------------------------------

/**
 * Fetch auth state and all application state values from the polling endpoint.
 *
 * Returns { user, edition, version, stateValues } or null on failure.
 */
async function fetchAuthAndState() {
  const csrfToken = ensureCSRFToken();

  const response = await fetch(STATE_POLL_URL, {
    method: 'GET',
    credentials: 'include',
    headers: {
      Accept: 'application/json',
      [CSRF_COOKIE_NAME]: csrfToken,
      'X-Astrum-UI': 'true',
    },
  });

  if (!response.ok) {
    return null;
  }

  const result = await response.json();
  const data = result?.data?.data;

  if (!data || typeof data !== 'object') {
    return null;
  }

  // Unwrap StateValueXO wrappers: { hash, value } → value
  const stateValues = {};
  for (const [key, wrapper] of Object.entries(data)) {
    if (wrapper && typeof wrapper === 'object' && 'value' in wrapper) {
      stateValues[key] = wrapper.value;
    } else {
      stateValues[key] = wrapper;
    }
  }

  // Extract user
  const userValue = stateValues.user || null;
  const user = userValue
    ? {
        id: userValue.id,
        authenticated: userValue.authenticated,
        administrator: userValue.administrator,
        authenticatedRealms: userValue.authenticatedRealms || [],
      }
    : null;

  // Extract edition and version from status
  const statusValue = stateValues.status || {};
  let edition = statusValue.edition || null;
  const version = statusValue.version || null;

  // Fallback: try license for edition
  if (!edition && stateValues.license?.edition) {
    edition = stateValues.license.edition;
  }

  // Default to PRO if authenticated but edition unknown
  if (!edition && user?.authenticated) {
    edition = 'PRO';
  }

  return { user, edition, version, stateValues };
}

/**
 * Fetch permissions via ExtDirect RPC.
 *
 * Returns a Set of lowercase permission strings, or empty Set on failure.
 */
async function fetchPermissions() {
  const csrfToken = ensureCSRFToken();

  try {
    const response = await fetch(EXTDIRECT_URL, {
      method: 'POST',
      credentials: 'include',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
        [CSRF_COOKIE_NAME]: csrfToken,
        'X-Astrum-UI': 'true',
      },
      body: JSON.stringify({
        action: 'rapture_Security',
        method: 'getPermissions',
        data: null,
        type: 'rpc',
        tid: Date.now(),
      }),
    });

    if (!response.ok) {
      return new Set();
    }

    const result = await response.json();
    const data = result?.result?.data;

    if (Array.isArray(data)) {
      return new Set(data.map((p) => p.id.toLowerCase()));
    }
  } catch (e) {
    console.warn('[RestBootstrap] Failed to fetch permissions:', e);
  }

  return new Set();
}

// ---------------------------------------------------------------------------
// Seed window.AS Globals
// ---------------------------------------------------------------------------

/**
 * Store REST-fetched data in a SEPARATE namespace (window.__asRestBootstrap).
 *
 * IMPORTANT: We do NOT touch window.AS.State or window.AS.Permissions.
 * Those are owned by ExtJS. Overwriting them breaks ExtJS initialization
 * (e.g., AS.State.setBrowserSupported is not a function).
 *
 * Instead, ExtJS.js fallbacks are enhanced to read from __asRestBootstrap
 * when the real AS.State/AS.Permissions aren't available yet.
 */
function seedRestData(authState, permissions) {
  if (typeof window === 'undefined') return;

  const { user, edition, version, stateValues } = authState;

  // Ensure status is in stateValues
  if (!stateValues.status) {
    stateValues.status = { edition, version };
  }

  // Store in separate namespace - ExtJS.js reads from here as fallback
  window.__asRestBootstrap = {
    user,
    edition,
    version,
    stateValues,
    permissions,
    checkPermission: (permission) => checkPermission(permissions, permission),
  };

  console.info(
    `[RestBootstrap] Stored REST data: user=${user?.id || 'anonymous'}, edition=${edition}, permissions=${permissions.size}`
  );
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Bootstrap the application from REST endpoints.
 *
 * Awaits only auth/state so React can render immediately. Permissions are
 * fetched in parallel but resolved asynchronously — the app seeds with an
 * empty permission set first, then updates window.__asRestBootstrap.permissions
 * once the fetch completes (ASTRUM-52583: getPermissions can take 20s on cold
 * cache; blocking render on it produces a blank screen for that entire period).
 *
 * On failure (e.g., 401 not authenticated), seeds with empty/anonymous state
 * so the app can still render (and redirect to login if needed).
 */
export async function bootstrapFromREST() {
  // Start permissions fetch immediately (parallel) but do NOT await it yet
  const permissionsPromise = fetchPermissions();

  try {
    const authState = await fetchAuthAndState();

    // Seed with empty permissions so React renders right away
    if (authState) {
      seedRestData(authState, new Set());
    } else {
      seedRestData(
        { user: null, edition: null, version: null, stateValues: {} },
        new Set()
      );
    }

    // Update permissions asynchronously once the slow fetch completes
    permissionsPromise
      .then((permissions) => {
        if (window.__asRestBootstrap) {
          window.__asRestBootstrap.permissions = permissions;
          window.__asRestBootstrap.checkPermission = (permission) =>
            checkPermission(permissions, permission);
          console.info(
            `[RestBootstrap] Permissions updated asynchronously: ${permissions.size} entries`
          );
          // Notify any React components that rendered with the empty initial set so they
          // can re-check permissions now that the real set is available.
          window.dispatchEvent(
            new CustomEvent('as-rest-permissions-loaded', {detail: {count: permissions.size}})
          );
        }
      })
      .catch((e) => {
        console.warn('[RestBootstrap] Async permissions fetch failed:', e);
      });
  } catch (error) {
    console.error('[RestBootstrap] Bootstrap failed:', error);
    seedRestData(
      { user: null, edition: null, version: null, stateValues: {} },
      new Set()
    );
  }
}
