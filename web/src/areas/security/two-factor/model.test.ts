import { describe, expect, it } from 'vitest';
import { ApiError } from '../../../api';
import { makeTranslate } from '../../../i18n';
import type { TwoFactorPolicy } from './api';
import {
  type PolicyForm,
  secondFactorBadge,
  serverProblems,
  toForm,
  toPolicy,
  validatePolicy,
} from './model';

const t = makeTranslate('en');
const POLICY: TwoFactorPolicy = {
  totp: true,
  passkeys: false,
  required: { everyone: false, administrators: true, roles: ['dev'] },
  graceDays: 7,
  requiredSince: null,
  relyingParty: { id: null, origins: [] },
};

describe('second factor model', () => {
  it('derives the list badge from enrolment and requirement', () => {
    expect(secondFactorBadge({ enrolled: true, required: true, requirement: 'required' })).toBe('setUp');
    expect(secondFactorBadge({ enrolled: false, required: true, requirement: 'policy' })).toBe('required');
    expect(secondFactorBadge({ enrolled: false, required: false, requirement: 'exempt' })).toBe('exempt');
    expect(secondFactorBadge({ enrolled: false, required: false, requirement: 'policy' })).toBeNull();
    expect(secondFactorBadge(undefined)).toBeNull();
  });

  it('maps server field ids onto the controls and keeps the rest as messages', () => {
    const error = new ApiError(400, 'bad', {
      fieldErrors: [
        { id: 'required.roles', message: 'the role dev does not exist' },
        { id: 'relyingParty.id', message: 'must be a host name' },
        { id: 'required', message: 'must be an object' },
      ],
    });
    expect(serverProblems(error)).toEqual({
      fields: { roles: 'the role dev does not exist', relyingPartyId: 'must be a host name' },
      unmatched: ['must be an object'],
    });
  });

  it('sends an empty relying party id as null and checks the grace period range', () => {
    const values: PolicyForm = { ...toForm(POLICY), relyingPartyId: '  ' };
    expect(toPolicy(values).relyingParty).toEqual({ id: null, origins: [] });
    expect(validatePolicy(t, values)).toEqual({});
    expect(validatePolicy(t, { ...values, graceDays: 91 })).toEqual({
      graceDays: 'Enter a whole number of days from 0 to 90',
    });
  });
});
