export const LOGIN_PATH = '/login';

export function safeReturnTo(value: string | null | undefined): string {
  if (!value?.startsWith('/') || value.startsWith('//') || value.startsWith('/\\')) {
    return '/';
  }
  if (value === LOGIN_PATH || value.startsWith(`${LOGIN_PATH}?`) || value.startsWith(`${LOGIN_PATH}/`)) {
    return '/';
  }
  return value;
}

export function loginPath(returnTo?: string): string {
  const target = safeReturnTo(returnTo);
  return target === '/' ? LOGIN_PATH : `${LOGIN_PATH}?returnTo=${encodeURIComponent(target)}`;
}
