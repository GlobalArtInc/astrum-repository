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
  expiresAt: string;
  lastUsedAt: string | null;
}

export interface CreatedToken extends AccessToken {
  token: string;
}

export interface TokenValues {
  name: string;
  scope: Scope;
  days: number | null;
}

export const MIN_DAYS = 1;
export const MAX_DAYS = 3650;
export const DAY_PRESETS = [7, 30, 90, 365] as const;

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
  if (
    values.days === null ||
    !Number.isInteger(values.days) ||
    values.days < MIN_DAYS ||
    values.days > MAX_DAYS
  ) {
    errors.days = t('user.tokens.daysRule', { min: MIN_DAYS, max: MAX_DAYS });
  }
  return errors;
}

export function toTokenRequest(values: TokenValues) {
  return { name: values.name.trim(), scope: values.scope, expiresInDays: values.days ?? 0 };
}

export function tokenExpired(token: Pick<AccessToken, 'expiresAt'>, now: Date): boolean {
  return new Date(token.expiresAt).getTime() <= now.getTime();
}

export function needsPasswordSignIn(error: unknown): boolean {
  return error instanceof ApiError && error.status === 403 && /password/i.test(error.message);
}

export function basicExample(baseUrl: string, userId: string, token: string): string {
  const base = baseUrl.replace(/\/+$/, '');
  const quoted = `${userId}:${token}`.replace(/'/g, `'\\''`);
  return `curl -u '${quoted}' ${base}/repository/<repository>/<path>`;
}
