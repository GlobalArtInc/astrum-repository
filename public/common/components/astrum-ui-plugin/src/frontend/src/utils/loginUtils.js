

/**
 * Parses a Retry-After header value (RFC 9110 §10.2.3).
 * Accepts either a delay-seconds integer or an HTTP-date string.
 * Returns null if the header is absent or unparseable.
 */
export function parseRetryAfter(header) {
  if (!header) return null;
  const seconds = Number(header);
  if (!isNaN(seconds)) return Math.max(1, seconds);
  const delta = Math.ceil((new Date(header) - Date.now()) / 1000);
  return isFinite(delta) && delta > 0 ? delta : null;
}

/**
 * sessionStorage key set when a user explicitly switches to the Classic (Heritage) UI.
 * Mirrors the key used by the global header's "Switch to Classic UI" control.
 */
export const USER_REQUESTED_LEGACY_KEY = 'user_requested_legacy';

/** ui-router state name for the Astrum One (Preview) UI dashboard. */
export const PREVIEW_WELCOME_ROUTE = 'preview.browse.welcome';

/** ui-router state name for the Classic UI dashboard. */
export const CLASSIC_WELCOME_ROUTE = 'browse.welcome';

/** URL hash form of the Preview welcome route, as the SSO SAML/OIDC backends expect (Base64-decoded). */
export const PREVIEW_WELCOME_HASH = '#preview/browse/welcome';

/** localStorage key holding the interface the user used last in this browser: 'preview' or 'classic'. */
export const LAST_USED_UI_KEY = 'astrum-last-ui';

/**
 * The interface the user used last in this browser ('preview' or 'classic'), or null when they have not
 * used either yet or the browser keeps no storage.
 *
 * @return {?string}
 */
export function getLastUsedUi() {
  try {
    const value = localStorage.getItem(LAST_USED_UI_KEY);
    return value === 'preview' || value === 'classic' ? value : null;
  }
  catch {
    return null;
  }
}

/**
 * Remembers the interface the user is using so the next visit of the dashboard opens it again.
 *
 * @param {string} ui - 'preview' or 'classic'
 */
export function setLastUsedUi(ui) {
  try {
    localStorage.setItem(LAST_USED_UI_KEY, ui);
  }
  catch {
    // No storage: the admin default decides.
  }
}

/**
 * Resolves which welcome/dashboard route a user should land on when there is no
 * explicit returnTo target. A user allowed into Preview who has not chosen the Classic UI
 * in this browser session lands on the interface they used last in this browser; a user
 * who has used neither yet lands on Preview when the admin has enabled "Default to
 * Astrum One UI".
 *
 * @param {Object} opts
 * @param {boolean} opts.defaultToPreviewUi - the defaultToPreviewUi admin setting
 * @param {boolean} opts.canAccessPreviewUi - whether this user may use Preview
 *   (loggedInEnabled when authenticated, anonymousEnabled when anonymous)
 * @param {boolean} opts.userRequestedLegacy - user opted into Classic this session
 * @param {?string} [opts.lastUsedUi] - 'preview' or 'classic', read from this browser when omitted
 * @return {string} a ui-router state name
 */
export function resolveDefaultLandingRoute({
  defaultToPreviewUi,
  canAccessPreviewUi,
  userRequestedLegacy,
  lastUsedUi = getLastUsedUi()
}) {
  if (!canAccessPreviewUi || userRequestedLegacy) {
    return CLASSIC_WELCOME_ROUTE;
  }
  if (lastUsedUi) {
    return lastUsedUi === 'preview' ? PREVIEW_WELCOME_ROUTE : CLASSIC_WELCOME_ROUTE;
  }
  return defaultToPreviewUi ? PREVIEW_WELCOME_ROUTE : CLASSIC_WELCOME_ROUTE;
}

/**
 * True when a decoded returnTo hash/path targets the Preview UI
 * (e.g. '#preview/browse/welcome' or '#/preview/...').
 *
 * @param {string} hash - a URL hash or path (with or without a leading '#'/'/')
 * @return {boolean}
 */
export function isPreviewHash(hash) {
  if (!hash) {
    return false;
  }
  const path = String(hash).replace(/^#/, '').replace(/^\//, '');
  return path === 'preview' || path.startsWith('preview/');
}

/**
 * True when a returnTo param targets the Preview UI. Accepts either the Base64-encoded
 * '#...' hash produced by the login redirects, or a raw '#...'/'/...' value. Used to
 * decide whether a fresh login should honor the returnTo or override a Classic
 * destination with the Preview dashboard (ASTRUM-53957).
 *
 * @param {string} returnToParam - Base64-encoded or raw returnTo value
 * @return {boolean}
 */
/**
 * True when a decoded returnTo hash/path targets a Classic landing page (the dashboard or
 * app root) — the only Classic destinations the "Default to Astrum One UI" soft default
 * overrides. Classic deep links are preserved. A missing hash counts as a landing page (no
 * explicit destination → the default applies). Mirrors the isLandingPage gate in
 * GlobalHeaderRadix so login and the header share one soft-default contract (ASTRUM-53957).
 *
 * @param {string} hash - a URL hash or path (with or without a leading '#'/'/')
 * @return {boolean}
 */
export function isClassicLandingHash(hash) {
  if (!hash) {
    return true;
  }
  const path = String(hash).replace(/^#/, '').replace(/^\//, '');
  return path === '' || path === 'browse/welcome';
}

/**
 * Decodes a returnTo param (Base64-encoded '#...' hash, or a raw '#...'/'/...' value) to its
 * hash/path form. Returns '' when there is no returnTo.
 */
function decodeReturnTo(returnToParam) {
  if (!returnToParam) {
    return '';
  }
  let value = String(returnToParam);
  try {
    const decoded = atob(value);
    if (decoded.startsWith('#') || decoded.startsWith('/')) {
      value = decoded;
    }
  }
  catch {
    // Not Base64 — treat the original value as a raw hash/path.
  }
  return value;
}

/**
 * True when a returnTo param targets the Preview UI. Accepts Base64-encoded or raw values.
 */
export function returnToTargetsPreview(returnToParam) {
  return isPreviewHash(decodeReturnTo(returnToParam));
}

/**
 * True when a returnTo param is missing or targets a Classic landing page. Accepts
 * Base64-encoded or raw values. Used to decide whether a fresh login should override the
 * returnTo with the Preview dashboard (landing pages only) while preserving Classic deep
 * links (ASTRUM-53957).
 */
export function returnToTargetsClassicLanding(returnToParam) {
  return isClassicLandingHash(decodeReturnTo(returnToParam));
}
