import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../i18n';
import { expiryTone, idProblem, validateNewCustomer } from './model';

const t = makeTranslate('en');
const DAY_MS = 24 * 60 * 60 * 1000;
const NOW = Date.parse('2026-11-11T12:00:00Z');

describe('the expiry tone of a customer or token', () => {
  it('warns within 30 days, is an error once past and stays quiet otherwise', () => {
    expect(expiryTone(new Date(NOW + 10 * DAY_MS).toISOString(), NOW)).toBe('warning');
    expect(expiryTone(new Date(NOW - DAY_MS).toISOString(), NOW)).toBe('danger');
    expect(expiryTone(new Date(NOW + 90 * DAY_MS).toISOString(), NOW)).toBeUndefined();
    expect(expiryTone(null, NOW)).toBeUndefined();
  });
});

describe('the customer id', () => {
  it('accepts lower-case slugs and rejects anything else', () => {
    expect(idProblem(t, 'acme-2')).toBeUndefined();
    expect(idProblem(t, 'Acme')).toBe('Use 1 to 64 lower-case letters, digits and hyphens');
    expect(idProblem(t, 'a'.repeat(65))).toBeDefined();
    expect(
      validateNewCustomer(t, { id: 'acme', name: ' ', reference: '', note: '', disabled: false }),
    ).toEqual({
      name: 'Enter a name',
    });
  });
});
