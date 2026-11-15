import { type ApiError, fieldName } from '../../../api';
import type { Translate } from '../../../i18n';
import type { Errors } from '../../../ui';
import type { SecondFactorRequirement, TwoFactorPolicy, UserSecondFactor } from './api';

export const GRACE_DAYS_MAX = 90;

export type SecondFactorBadge = 'setUp' | 'required' | 'exempt';

export interface PolicyForm {
  totp: boolean;
  passkeys: boolean;
  everyone: boolean;
  administrators: boolean;
  roles: string[];
  graceDays: number | null;
  requiredSince: string | null;
  relyingPartyId: string;
  origins: string[];
}

export interface ServerProblems {
  fields: Errors<PolicyForm>;
  unmatched: string[];
}

const REQUIREMENTS: readonly SecondFactorRequirement[] = ['policy', 'required', 'exempt'];

const FIELD_BY_SERVER_ID: Record<string, keyof PolicyForm> = {
  totp: 'totp',
  passkeys: 'passkeys',
  graceDays: 'graceDays',
  'required.roles': 'roles',
  'relyingParty.id': 'relyingPartyId',
  'relyingParty.origins': 'origins',
};

export function isRequirement(value: string): value is SecondFactorRequirement {
  return (REQUIREMENTS as readonly string[]).includes(value);
}

export function secondFactorBadge(factor: UserSecondFactor | undefined): SecondFactorBadge | null {
  if (factor === undefined) {
    return null;
  }
  if (factor.enrolled) {
    return 'setUp';
  }
  if (factor.required) {
    return 'required';
  }
  return factor.requirement === 'exempt' ? 'exempt' : null;
}

export function toForm(policy: TwoFactorPolicy): PolicyForm {
  return {
    totp: policy.totp,
    passkeys: policy.passkeys,
    everyone: policy.required.everyone,
    administrators: policy.required.administrators,
    roles: policy.required.roles,
    graceDays: policy.graceDays,
    requiredSince: policy.requiredSince,
    relyingPartyId: policy.relyingParty.id ?? '',
    origins: policy.relyingParty.origins,
  };
}

export function toPolicy(values: PolicyForm): TwoFactorPolicy {
  return {
    totp: values.totp,
    passkeys: values.passkeys,
    required: {
      everyone: values.everyone,
      administrators: values.administrators,
      roles: values.roles,
    },
    graceDays: values.graceDays ?? 0,
    requiredSince: values.requiredSince,
    relyingParty: {
      id: values.relyingPartyId.trim() || null,
      origins: values.origins,
    },
  };
}

export function validatePolicy(t: Translate, values: PolicyForm): Errors<PolicyForm> {
  const days = values.graceDays;
  if (days !== null && Number.isInteger(days) && days >= 0 && days <= GRACE_DAYS_MAX) {
    return {};
  }
  return { graceDays: t('security.twoFactor.graceDaysBad') };
}

export function noMethodWarning(values: PolicyForm): boolean {
  const someoneRequired = values.everyone || values.administrators || values.roles.length > 0;
  return !values.totp && !values.passkeys && someoneRequired;
}

export function serverProblems(error: ApiError): ServerProblems {
  const fields: Errors<PolicyForm> = {};
  const unmatched: string[] = [];
  for (const problem of error.fieldErrors) {
    const name = FIELD_BY_SERVER_ID[fieldName(problem.id)];
    if (name === undefined) {
      unmatched.push(problem.message);
    } else if (fields[name] === undefined) {
      fields[name] = problem.message;
    }
  }
  return { fields, unmatched };
}
