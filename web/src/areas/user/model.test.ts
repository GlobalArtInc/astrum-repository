import { describe, expect, it } from 'vitest';
import { ApiError } from '../../api';
import { makeTranslate } from '../../i18n';
import { type AgentGrant, filterGrants } from './agents';
import {
  type AccessToken,
  filterTokens,
  needsPasswordSignIn,
  passwordFailure,
  sortNewestFirst,
  toAccountRequest,
  tokenExpired,
  toTokenRequest,
  usageExamples,
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
  it('needs a name and sends the lifetime, or null for a token that never expires', () => {
    expect(validateToken(t, { name: ' ', scope: 'READ', days: null })).toEqual({ name: 'Fill this in' });
    expect(validateToken(t, { name: 'x'.repeat(201), scope: 'READ', days: 7 }).name).toBeDefined();
    expect(validateToken(t, { name: 'ci', scope: 'WRITE', days: 365 })).toEqual({});
    expect(toTokenRequest({ name: ' ci ', scope: 'WRITE', days: 30 })).toEqual({
      name: 'ci',
      scope: 'WRITE',
      expiresInDays: 30,
    });
    expect(toTokenRequest({ name: 'ci', scope: 'READ', days: null }).expiresInDays).toBeNull();
  });

  it('tells expired tokens and the password rule apart', () => {
    const now = new Date('2026-10-12T12:00:00Z');
    expect(tokenExpired({ expiresAt: '2026-10-12T11:59:59Z' }, now)).toBe(true);
    expect(tokenExpired({ expiresAt: '2026-10-13T00:00:00Z' }, now)).toBe(false);
    expect(tokenExpired({ expiresAt: null }, now)).toBe(false);
    expect(
      needsPasswordSignIn(new ApiError(403, 'access tokens are managed after signing in with a password')),
    ).toBe(true);
    expect(needsPasswordSignIn(new ApiError(403, 'you do not have the permission this needs'))).toBe(false);
  });

  it('searches tokens by any word in name, user, hint or scope and lists the newest first', () => {
    const tokens: AccessToken[] = [
      {
        id: '1',
        userId: 'admin',
        name: 'CI build',
        hint: 'astrum_abcd',
        scope: 'READ',
        createdAt: '2026-10-01T10:00:00Z',
        expiresAt: null,
        lastUsedAt: null,
      },
      {
        id: '2',
        userId: 'jane',
        name: 'Laptop',
        hint: 'astrum_wxyz',
        scope: 'WRITE',
        createdAt: '2026-10-03T10:00:00Z',
        expiresAt: null,
        lastUsedAt: null,
      },
      {
        id: '3',
        userId: 'admin',
        name: 'laptop backup',
        hint: 'astrum_efgh',
        scope: 'READ',
        createdAt: '2026-10-02T10:00:00Z',
        expiresAt: null,
        lastUsedAt: null,
      },
    ];
    expect(filterTokens(tokens, '  ')).toBe(tokens);
    expect(filterTokens(tokens, 'LAPTOP').map((token) => token.id)).toEqual(['2', '3']);
    expect(filterTokens(tokens, 'laptop jane').map((token) => token.id)).toEqual(['2']);
    expect(filterTokens(tokens, 'admin ci').map((token) => token.id)).toEqual(['1']);
    expect(filterTokens(tokens, 'jane').map((token) => token.id)).toEqual(['2']);
    expect(filterTokens(tokens, 'write').map((token) => token.id)).toEqual(['2']);
  });

  it('sorts tokens newest first without touching the input', () => {
    const tokens = [
      { createdAt: '2026-10-01T10:00:00Z', id: '1' },
      { createdAt: '2026-10-03T10:00:00Z', id: '2' },
      { createdAt: '2026-10-02T10:00:00Z', id: '3' },
    ];
    expect(sortNewestFirst(tokens).map((token) => token.id)).toEqual(['2', '3', '1']);
    expect(tokens.map((token) => token.id)).toEqual(['1', '2', '3']);
  });

  it('searches connected agents by client name', () => {
    const grants: AgentGrant[] = [
      {
        id: 'g1',
        clientName: 'Claude Code',
        scope: 'WRITE',
        createdAt: '2026-10-19T10:00:00Z',
        lastUsedAt: null,
        expiresAt: '2026-11-18T10:00:00Z',
      },
      {
        id: 'g2',
        clientName: 'Cursor',
        scope: 'READ',
        createdAt: '2026-10-20T10:00:00Z',
        lastUsedAt: null,
        expiresAt: '2026-11-19T10:00:00Z',
      },
    ];
    expect(filterGrants(grants, '')).toBe(grants);
    expect(filterGrants(grants, 'claude code').map((grant) => grant.id)).toEqual(['g1']);
    expect(filterGrants(grants, 'CURSOR').map((grant) => grant.id)).toEqual(['g2']);
  });

  it('shows how each client uses the token as the password', () => {
    const examples = usageExamples('http://repo.example:8081/', 'jane', 'astrum_x');
    expect(examples.docker).toBe("echo 'astrum_x' | docker login repo.example:8081 -u jane --password-stdin");
    expect(examples.podman).toBe("echo 'astrum_x' | podman login repo.example:8081 -u jane --password-stdin");
    expect(examples.npm).toBe(
      `registry=http://repo.example:8081/repository/<repository>/\n//repo.example:8081/repository/<repository>/:_auth=${btoa('jane:astrum_x')}`,
    );
    expect(examples.maven).toContain('<username>jane</username>\n  <password>astrum_x</password>');
    expect(examples.pip).toBe(
      'pip install --index-url http://jane:astrum_x@repo.example:8081/repository/<repository>/simple/ <package>',
    );
    expect(examples.curl).toBe(
      'curl -u jane:astrum_x -O http://repo.example:8081/repository/<repository>/<path>',
    );
  });
});
