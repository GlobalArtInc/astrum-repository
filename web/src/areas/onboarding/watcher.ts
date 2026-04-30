import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { CSRF_COOKIE } from '../../api';
import { useSession } from '../../auth/state';
import { readCookie } from '../../i18n';
import { readStored, writeStored } from '../../ui';

export const ONBOARDING_PATH = '/onboarding';
export const OFFERED_KEY = 'astrum.onboarding.offered';
export const ADMINISTRATOR = 'astrum:*';

export function fingerprint(text: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash.toString(16).padStart(8, '0');
}

export function sessionMarker(userId: string, cookies: string = document.cookie): string {
  return fingerprint(`${userId}\u0000${readCookie(CSRF_COOKIE, cookies) ?? ''}`);
}

export function onboardingTarget(returnTo: string): string {
  return returnTo === '/' || returnTo.startsWith(ONBOARDING_PATH)
    ? ONBOARDING_PATH
    : `${ONBOARDING_PATH}?returnTo=${encodeURIComponent(returnTo)}`;
}

export function shouldOffer(options: {
  signedIn: boolean;
  administrator: boolean;
  passwordFile: string | null;
  pathname: string;
  offered: string | null;
  marker: string;
}): boolean {
  return (
    options.signedIn &&
    options.administrator &&
    options.passwordFile !== null &&
    !options.pathname.startsWith(ONBOARDING_PATH) &&
    options.offered !== options.marker
  );
}

export function OnboardingWatcher() {
  const { user, state, can } = useSession();
  const navigate = useNavigate();
  const location = useLocation();
  const administrator = can(ADMINISTRATOR);
  const userId = user?.userId ?? null;
  const passwordFile = state.adminPasswordFile;
  const { pathname, search, hash } = location;
  useEffect(() => {
    if (userId === null) {
      return;
    }
    const marker = sessionMarker(userId);
    const offer = shouldOffer({
      signedIn: true,
      administrator,
      passwordFile,
      pathname,
      offered: readStored(OFFERED_KEY),
      marker,
    });
    if (offer) {
      writeStored(OFFERED_KEY, marker);
      navigate(onboardingTarget(pathname + search + hash), { replace: true });
    }
  }, [userId, administrator, passwordFile, pathname, search, hash, navigate]);
  return null;
}
