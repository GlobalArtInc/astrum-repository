import { path, withQuery } from '../../../api';
import type { Translate } from '../../../i18n';
import type { Tone } from '../../../ui';
import { DEFAULT_SOURCE, USERS, type User, type UserSource, type UserStatus } from '../api';
import { validateId } from '../roles/model';

export const STATUSES: UserStatus[] = ['active', 'disabled', 'locked', 'changepassword'];
export const MIN_PASSWORD = 8;

const STATUS_LABELS: Record<UserStatus, string> = {
  active: 'security.userStatus.active',
  disabled: 'security.userStatus.disabled',
  locked: 'security.userStatus.locked',
  changepassword: 'security.userStatus.changepassword',
};

const SOURCE_LABELS: Record<string, string> = {
  default: 'security.userSource.default',
  LDAP: 'security.userSource.ldap',
  OAuth2: 'security.userSource.oauth2',
};

export function statusLabel(t: Translate, status: string): string {
  const key = STATUS_LABELS[status as UserStatus];
  return key ? t(key) : status;
}

export function sourceLabel(t: Translate, source: string): string {
  const key = SOURCE_LABELS[source];
  return key ? t(key) : source;
}

export interface UserValues {
  userId: string;
  firstName: string;
  lastName: string;
  emailAddress: string;
  password: string;
  confirm: string;
  status: UserStatus;
  roles: string[];
}

export const EMPTY_USER: UserValues = {
  userId: '',
  firstName: '',
  lastName: '',
  emailAddress: '',
  password: '',
  confirm: '',
  status: 'active',
  roles: [],
};

export function fromUser(user: User): UserValues {
  return {
    userId: user.userId,
    firstName: user.firstName ?? '',
    lastName: user.lastName ?? '',
    emailAddress: user.emailAddress ?? '',
    password: '',
    confirm: '',
    status: user.status,
    roles: [...user.roles],
  };
}

export function toCreateRequest(values: UserValues) {
  return {
    userId: values.userId.trim(),
    firstName: values.firstName.trim(),
    lastName: values.lastName.trim(),
    emailAddress: values.emailAddress.trim(),
    password: values.password,
    status: values.status,
    roles: [...values.roles],
  };
}

export function toUpdateRequest(values: UserValues, user: User) {
  const local = user.source === DEFAULT_SOURCE;
  return {
    userId: user.userId,
    source: user.source,
    firstName: local ? values.firstName.trim() : user.firstName,
    lastName: local ? values.lastName.trim() : user.lastName,
    emailAddress: local ? values.emailAddress.trim() : user.emailAddress,
    status: local ? values.status : user.status,
    roles: [...values.roles],
  };
}

const EMAIL = /^[^\s@]+@[^\s@]+$/;

export function emailProblem(t: Translate, value: string): string | undefined {
  if (value.trim() === '') {
    return t('security.form.required');
  }
  return EMAIL.test(value.trim()) ? undefined : t('security.form.emailRule');
}

export function passwordProblems(
  t: Translate,
  password: string,
  confirm: string,
): { password?: string; confirm?: string } {
  const problems: { password?: string; confirm?: string } = {};
  if (password === '') {
    problems.password = t('security.form.required');
  } else if ([...password].length < MIN_PASSWORD) {
    problems.password = t('security.form.passwordRule', { count: MIN_PASSWORD });
  }
  if (confirm !== password) {
    problems.confirm = t('security.form.passwordMismatch');
  }
  return problems;
}

export function validateUser(
  t: Translate,
  values: UserValues,
  options: { creating: boolean; local: boolean },
): Partial<Record<keyof UserValues, string>> {
  const errors: Partial<Record<keyof UserValues, string>> = {};
  if (options.creating) {
    const id = validateId(t, values.userId);
    if (id) {
      errors.userId = id;
    }
    Object.assign(errors, passwordProblems(t, values.password, values.confirm));
  }
  if (options.local) {
    if (values.firstName.trim() === '') {
      errors.firstName = t('security.form.required');
    }
    if (values.lastName.trim() === '') {
      errors.lastName = t('security.form.required');
    }
    const email = emailProblem(t, values.emailAddress);
    if (email) {
      errors.emailAddress = email;
    }
  }
  return errors;
}

export function fullName(user: Pick<User, 'firstName' | 'lastName'>): string {
  return [user.firstName, user.lastName]
    .map((part) => (part ?? '').trim())
    .filter(Boolean)
    .join(' ');
}

export const STATUS_TONES: Record<UserStatus, Tone> = {
  active: 'success',
  disabled: 'neutral',
  locked: 'danger',
  changepassword: 'warning',
};

export const FALLBACK_SOURCES: UserSource[] = [{ id: DEFAULT_SOURCE, name: 'AstrumAuthenticatingRealm' }];

export function deleteQuery(user: User, sources: readonly UserSource[]): string {
  if (user.source === DEFAULT_SOURCE) {
    return `${USERS}${path`/${user.userId}`}`;
  }
  const realm = sources.find((source) => source.id === user.source)?.name;
  return withQuery(`${USERS}${path`/${user.userId}`}`, { realm });
}
