

import { withoutPreviewPrefix } from './previewHeritageNavigation';

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
 * sessionStorage key that earlier releases set when a user switched to the Classic (Heritage) UI.
 * The Classic UI is retired, so the key is only cleared and never honored.
 */
export const USER_REQUESTED_LEGACY_KEY = 'user_requested_legacy';

/** ui-router state name for the Astrum One (Preview) UI dashboard. */
export const PREVIEW_WELCOME_ROUTE = 'preview.browse.welcome';

/** URL hash form of the Preview welcome route, as the SSO SAML/OIDC backends expect (Base64-decoded). */
export const PREVIEW_WELCOME_HASH = '#browse/welcome';

/**
 * Resolves which welcome/dashboard route a user should land on when there is no
 * explicit returnTo target. The Classic UI is retired, so everyone lands on the Astrum One
 * dashboard; a Classic choice saved in this browser by an earlier release (the session flag
 * or the interface used last) no longer counts.
 *
 * @return {string} a ui-router state name
 */
export function resolveDefaultLandingRoute() {
  return PREVIEW_WELCOME_ROUTE;
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
 * The Astrum One form of a decoded returnTo hash/path ('#...' or '/...'). A destination under the
 * `preview/` prefix Astrum One pages used to have loses the prefix; any other destination is kept, and
 * the router opens the Astrum One page of a Classic one, such as a bookmark or a link in an email that
 * sent the user to the sign-in page. The leading '#', '/' or '#/' is preserved.
 *
 * @param {string} hash - a decoded returnTo value
 * @return {string}
 */
export function toPreviewReturnTo(hash) {
  const [, prefix, path] = /^(#?\/?)([\s\S]*)$/.exec(String(hash ?? ''));
  const astrumOnePath = withoutPreviewPrefix(path);
  return `${prefix}${astrumOnePath ?? path}`;
}

/**
 * The Astrum One form of a returnTo param (Base64-encoded '#...' hash, or a raw '#...'/'/...' value),
 * Base64-encoded as the SSO SAML/OIDC backends expect. A missing returnTo stays missing, since the
 * application root opens the Astrum One dashboard; a value that is not a hash or path is returned
 * unchanged.
 *
 * @param {?string} returnToParam
 * @return {?string}
 */
export function toPreviewReturnToParam(returnToParam) {
  if (!returnToParam) {
    return returnToParam;
  }
  const decoded = decodeReturnTo(returnToParam);
  if (!decoded.startsWith('#') && !decoded.startsWith('/')) {
    return returnToParam;
  }
  try {
    return btoa(toPreviewReturnTo(decoded));
  }
  catch {
    return returnToParam;
  }
}
