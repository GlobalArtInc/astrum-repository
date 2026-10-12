import { describe, expect, it } from 'vitest';
import { ApiError } from '../../api';
import { makeTranslate } from '../../i18n';
import {
  basicExample,
  needsPasswordSignIn,
  passwordFailure,
  toAccountRequest,
  tokenExpired,
  toTokenRequest,
  validateAccount,
  validatePassword,
  validateToken,
} from './model';

const t = makeTranslate('en');

describe('the account forms', () => {
  it('trims the account and checks it', () => {
    expect(toAccountRequest({ firstName: ' A ', lastName: 'B ', email: ' a@b.c ' })).toEqual({
      firstName: 'A',
      lastName: 'B',
      email: 'a@b.c',
    });
    expect(validateAccount(t, { firstName: '', lastName: 'x', email: 'nope' })).toEqual({
      firstName: 'Fill this in',
      email: 'Enter an address such as jane@example.org',
    });
  });

  it('checks the new password against the rules and the current one', () => {
    expect(validatePassword(t, { currentPassword: '', newPassword: 'short', confirm: '' })).toEqual({
      currentPassword: 'Fill this in',
      newPassword: 'Use at least 8 characters',
      confirm: 'The passwords do not match',
    });
    expect(
      validatePassword(t, { currentPassword: 'same-word', newPassword: 'same-word', confirm: 'same-word' }),
    ).toEqual({
      newPassword: 'Choose a password other than the current one',
    });
  });

  it('explains a wrong current password, the pause and a refusal', () => {
    const wrong = passwordFailure(
      t,
      new ApiError(400, 'x', {
        fieldErrors: [{ id: 'currentPassword', message: 'the current password is wrong' }],
      }),
    ) as ApiError;
    expect(wrong.fieldErrors).toEqual([{ id: 'currentPassword', message: 'The current password is wrong' }]);
    expect((passwordFailure(t, new ApiError(429, 'x')) as Error).message).toContain(
      'Too many wrong passwords',
    );
    expect((passwordFailure(t, new ApiError(403, 'read-only token')) as Error).message).toBe(
      'The server refused: read-only token',
    );
    const other = new ApiError(400, 'x', { fieldErrors: [{ id: 'newPassword', message: 'too short' }] });
    expect(passwordFailure(t, other)).toBe(other);
  });
});

describe('access tokens', () => {
  it('needs a name and a whole number of days in range', () => {
    expect(validateToken(t, { name: ' ', scope: 'READ', days: null })).toEqual({
      name: 'Fill this in',
      days: 'Enter a whole number of days from 1 to 3,650',
    });
    expect(validateToken(t, { name: 'ci', scope: 'READ', days: 3651 }).days).toBeDefined();
    expect(validateToken(t, { name: 'ci', scope: 'READ', days: 1.5 }).days).toBeDefined();
    expect(validateToken(t, { name: 'ci', scope: 'WRITE', days: 3650 })).toEqual({});
    expect(toTokenRequest({ name: ' ci ', scope: 'WRITE', days: 30 })).toEqual({
      name: 'ci',
      scope: 'WRITE',
      expiresInDays: 30,
    });
  });

  it('tells expired tokens and the password rule apart', () => {
    const now = new Date('2026-10-12T12:00:00Z');
    expect(tokenExpired({ expiresAt: '2026-10-12T11:59:59Z' }, now)).toBe(true);
    expect(tokenExpired({ expiresAt: '2026-10-13T00:00:00Z' }, now)).toBe(false);
    expect(
      needsPasswordSignIn(new ApiError(403, 'access tokens are managed after signing in with a password')),
    ).toBe(true);
    expect(needsPasswordSignIn(new ApiError(403, 'you do not have the permission this needs'))).toBe(false);
  });

  it('builds a curl example with the token as the password', () => {
    expect(basicExample('http://repo.example/', "o'neil", 'astrum_x')).toBe(
      "curl -u 'o'\\''neil:astrum_x' http://repo.example/repository/<repository>/<path>",
    );
  });
});
