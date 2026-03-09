import { isLanguage, LANGUAGES, type Language } from './translate';

export const LANGUAGE_COOKIE = 'astrum-language';
const YEAR = 365 * 24 * 60 * 60;

export function readCookie(name: string, cookies: string = document.cookie): string | undefined {
  for (const pair of cookies.split(';')) {
    const [key, ...rest] = pair.trim().split('=');
    if (key === name) {
      return decodeURIComponent(rest.join('='));
    }
  }
  return undefined;
}

export function detectLanguage(
  cookies: string = document.cookie,
  preferred: readonly string[] = navigator.languages ?? [navigator.language],
): Language {
  const stored = readCookie(LANGUAGE_COOKIE, cookies);
  if (isLanguage(stored)) {
    return stored;
  }
  for (const tag of preferred) {
    const base = tag.toLowerCase().split('-')[0];
    if (isLanguage(base)) {
      return base;
    }
  }
  return LANGUAGES[0];
}

export function rememberLanguage(language: Language): void {
  document.cookie = `${LANGUAGE_COOKIE}=${language}; Path=/; Max-Age=${YEAR}; SameSite=Lax`;
}
