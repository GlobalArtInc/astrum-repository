import { type Choice, DAY } from './model';

export type Rule = 'vulnerability' | 'license' | 'signature' | 'cooldown';
export type Subject = 'advisory' | 'license' | 'component' | 'digest';
export type Expiry = '7d' | '30d' | '90d' | 'custom' | 'never';

export interface PolicyException {
  id: string;
  policy: string | null;
  rule: Rule;
  advisory: string | null;
  license: string | null;
  format: string | null;
  namespace: string | null;
  name: string | null;
  version: string | null;
  digest: string | null;
  reason: string;
  expiresAt: string | null;
  createdBy: string;
  createdAt: string;
}

export interface ExceptionForm {
  rule: Rule;
  policy: string;
  subject: Subject;
  advisory: string;
  license: string;
  format: string;
  namespace: string;
  name: string;
  version: string;
  digest: string;
  reason: string;
  expiry: Expiry;
  customExpiry: string;
}

export interface ExceptionRequest {
  policy?: string;
  rule: Rule;
  advisory?: string;
  license?: string;
  format?: string;
  namespace?: string;
  name?: string;
  version?: string;
  digest?: string;
  reason: string;
  expiresAt: string | null;
}

export const RULE_OPTIONS: Array<Choice<Rule>> = [
  { value: 'vulnerability', label: 'security.exceptions.rules.vulnerability' },
  { value: 'license', label: 'security.exceptions.rules.license' },
  { value: 'signature', label: 'security.exceptions.rules.signature' },
  { value: 'cooldown', label: 'security.exceptions.rules.cooldown' },
];

export const SUBJECT_OPTIONS: Array<Choice<Subject>> = [
  { value: 'advisory', label: 'security.exceptions.subjects.advisory' },
  { value: 'license', label: 'security.exceptions.subjects.license' },
  { value: 'component', label: 'security.exceptions.subjects.component' },
  { value: 'digest', label: 'security.exceptions.subjects.digest' },
];

export const EXPIRY_OPTIONS: Array<Choice<Expiry>> = [
  { value: '7d', label: 'security.exceptions.expiry.days7' },
  { value: '30d', label: 'security.exceptions.expiry.days30' },
  { value: '90d', label: 'security.exceptions.expiry.days90' },
  { value: 'custom', label: 'security.exceptions.expiry.custom' },
  { value: 'never', label: 'security.exceptions.expiry.never' },
];

export const EMPTY_EXCEPTION_FORM: ExceptionForm = {
  rule: 'vulnerability',
  policy: '',
  subject: 'advisory',
  advisory: '',
  license: '',
  format: '',
  namespace: '',
  name: '',
  version: '',
  digest: '',
  reason: '',
  expiry: '30d',
  customExpiry: '',
};

export function subjectsOf(rule: Rule): Subject[] {
  if (rule === 'license') {
    return ['license'];
  }
  if (rule === 'cooldown') {
    return ['component'];
  }
  if (rule === 'signature') {
    return ['component', 'digest'];
  }
  return ['advisory', 'component', 'digest'];
}

export function expiresAtOf(form: ExceptionForm, now: number): string | null {
  switch (form.expiry) {
    case '7d':
      return new Date(now + 7 * DAY).toISOString();
    case '30d':
      return new Date(now + 30 * DAY).toISOString();
    case '90d':
      return new Date(now + 90 * DAY).toISOString();
    case 'custom':
      return new Date(form.customExpiry).toISOString();
    case 'never':
      return null;
  }
}

export function customExpiryProblem(form: ExceptionForm, now: number): string | null {
  if (form.expiry !== 'custom') {
    return null;
  }
  const at = Date.parse(form.customExpiry);
  if (Number.isNaN(at)) {
    return 'security.exceptions.expiryInvalid';
  }
  return at > now ? null : 'security.exceptions.expiryPast';
}

export function targetFilled(form: ExceptionForm): boolean {
  switch (form.subject) {
    case 'advisory':
      return form.advisory.trim() !== '';
    case 'license':
      return form.license.trim() !== '';
    case 'digest':
      return form.digest.trim() !== '';
    case 'component':
      return [form.format, form.namespace, form.name, form.version].some((part) => part.trim() !== '');
  }
}

export function requestOf(form: ExceptionForm, now: number): ExceptionRequest {
  const body: ExceptionRequest = {
    rule: form.rule,
    reason: form.reason.trim(),
    expiresAt: expiresAtOf(form, now),
  };
  if (form.policy !== '') {
    body.policy = form.policy;
  }
  switch (form.subject) {
    case 'advisory':
      body.advisory = form.advisory.trim();
      break;
    case 'license':
      body.license = form.license.trim();
      break;
    case 'digest':
      body.digest = form.digest.trim();
      break;
    case 'component':
      if (form.format !== '') {
        body.format = form.format;
      }
      if (form.namespace.trim() !== '') {
        body.namespace = form.namespace.trim();
      }
      if (form.name.trim() !== '') {
        body.name = form.name.trim();
      }
      if (form.version.trim() !== '') {
        body.version = form.version.trim();
      }
      break;
  }
  return body;
}

export function expired(exception: PolicyException, now: number): boolean {
  return exception.expiresAt !== null && Date.parse(exception.expiresAt) <= now;
}

export function subjectText(exception: PolicyException): string {
  if (exception.advisory) {
    return exception.advisory;
  }
  if (exception.license) {
    return exception.license;
  }
  if (exception.digest) {
    return exception.digest;
  }
  const component = [exception.format, exception.namespace, exception.name].filter(Boolean).join('/');
  return exception.version ? `${component}@${exception.version}` : component;
}
