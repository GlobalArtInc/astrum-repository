import { ApiError, INTERNAL } from '../../api';
import type { Translate } from '../../i18n';

export const ACCOUNT = `${INTERNAL}/ui/user`;
export const ACCOUNT_PASSWORD = `${INTERNAL}/ui/user/password`;
export const TOKENS = `${INTERNAL}/access-tokens`;
export const MIN_PASSWORD = 8;
export const TOKEN_REALM = 'AccessTokenRealm';

export interface Account {
  userId: string;
  firstName: string;
  lastName: string;
  email: string;
  source: string;
  realm: string;
  status: string;
  roles: string[];
  readOnly: boolean;
}

export interface AccountValues {
  firstName: string;
  lastName: string;
  email: string;
}

export interface PasswordValues {
  currentPassword: string;
  newPassword: string;
  confirm: string;
}

export type Scope = 'READ' | 'WRITE';

export interface AccessToken {
  id: string;
  userId: string;
  name: string;
  hint: string;
  scope: Scope;
  createdAt: string;
  expiresAt: string | null;
  lastUsedAt: string | null;
}

export interface CreatedToken extends AccessToken {
  token: string;
}

export interface TokenValues {
  name: string;
  scope: Scope;
  days: Lifetime;
}

export const LIFETIMES = [7, 30, 90, 365, null] as const;
export type Lifetime = (typeof LIFETIMES)[number];

const EMAIL = /^[^\s@]+@[^\s@]+$/;

export function fromAccount(account: Account): AccountValues {
  return { firstName: account.firstName, lastName: account.lastName, email: account.email };
}

export function toAccountRequest(values: AccountValues): AccountValues {
  return { firstName: values.firstName.trim(), lastName: values.lastName.trim(), email: values.email.trim() };
}

export function validateAccount(
  t: Translate,
  values: AccountValues,
): Partial<Record<keyof AccountValues, string>> {
  const errors: Partial<Record<keyof AccountValues, string>> = {};
  if (values.firstName.trim() === '') {
    errors.firstName = t('user.form.required');
  }
  if (values.lastName.trim() === '') {
    errors.lastName = t('user.form.required');
  }
  if (values.email.trim() === '') {
    errors.email = t('user.form.required');
  } else if (!EMAIL.test(values.email.trim())) {
    errors.email = t('user.form.emailRule');
  }
  return errors;
}

export function validatePassword(
  t: Translate,
  values: PasswordValues,
): Partial<Record<keyof PasswordValues, string>> {
  const errors: Partial<Record<keyof PasswordValues, string>> = {};
  if (values.currentPassword === '') {
    errors.currentPassword = t('user.form.required');
  }
  if (values.newPassword === '') {
    errors.newPassword = t('user.form.required');
  } else if ([...values.newPassword].length < MIN_PASSWORD) {
    errors.newPassword = t('user.form.passwordRule', { count: MIN_PASSWORD });
  } else if (values.newPassword === values.currentPassword) {
    errors.newPassword = t('user.password.same');
  }
  if (values.confirm !== values.newPassword) {
    errors.confirm = t('user.form.passwordMismatch');
  }
  return errors;
}

export function passwordFailure(t: Translate, error: unknown): unknown {
  if (error instanceof ApiError) {
    if (error.status === 429) {
      return new Error(t('user.password.tooMany'));
    }
    if (error.status === 400 && error.fieldErrors.some((problem) => problem.id === 'currentPassword')) {
      return new ApiError(400, t('user.password.wrong'), {
        fieldErrors: [{ id: 'currentPassword', message: t('user.password.wrong') }],
      });
    }
    if (error.status === 403) {
      return new Error(t('user.form.refused', { message: error.message }));
    }
  }
  return error;
}

export function validateToken(t: Translate, values: TokenValues): Partial<Record<keyof TokenValues, string>> {
  const errors: Partial<Record<keyof TokenValues, string>> = {};
  if (values.name.trim() === '') {
    errors.name = t('user.form.required');
  } else if (values.name.trim().length > 200) {
    errors.name = t('user.tokens.nameTooLong');
  }
  return errors;
}

export function toTokenRequest(values: TokenValues) {
  return { name: values.name.trim(), scope: values.scope, expiresInDays: values.days };
}

export function tokenExpired(token: Pick<AccessToken, 'expiresAt'>, now: Date): boolean {
  return token.expiresAt !== null && new Date(token.expiresAt).getTime() <= now.getTime();
}

export function needsPasswordSignIn(error: unknown): boolean {
  return error instanceof ApiError && error.status === 403 && /password/i.test(error.message);
}

export type UsageKind = 'docker' | 'npm' | 'maven' | 'pip' | 'curl';

export const USAGE_KINDS: readonly UsageKind[] = ['docker', 'npm', 'maven', 'pip', 'curl'];

function base64(text: string): string {
  let binary = '';
  for (const byte of new TextEncoder().encode(text)) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

export function usageExamples(baseUrl: string, userId: string, token: string): Record<UsageKind, string> {
  const url = new URL(baseUrl.replace(/\/+$/, '') || 'http://localhost');
  const base = `${url.origin}${url.pathname.replace(/\/+$/, '')}`;
  const host = `${url.host}${url.pathname.replace(/\/+$/, '')}`;
  const repository = `${base}/repository/<repository>/`;
  return {
    docker: `echo '${token}' | docker login ${url.host} -u ${userId} --password-stdin`,
    npm: `registry=${repository}\n//${host}/repository/<repository>/:_auth=${base64(`${userId}:${token}`)}`,
    maven: `<server>\n  <id>astrum</id>\n  <username>${userId}</username>\n  <password>${token}</password>\n</server>`,
    pip: `pip install --index-url ${url.protocol}//${encodeURIComponent(userId)}:${token}@${host}/repository/<repository>/simple/ <package>`,
    curl: `curl -u ${userId}:${token} -O ${repository}<path>`,
  };
}
