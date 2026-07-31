

/** Dispatched when the shared REST client receives 401 so the shell can show SessionExpiryModal */
export const ASTRUM_SESSION_EXPIRED_EVENT = 'astrum:session-expired';

export function notifySessionExpiredFromRest(): void {
  if (typeof window === 'undefined') {
    return;
  }
  window.dispatchEvent(new CustomEvent(ASTRUM_SESSION_EXPIRED_EVENT));
}
