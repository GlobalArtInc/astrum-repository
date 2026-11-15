import { api, INTERNAL } from '../api';
import type { AssertionCredential, AssertionOptions } from './webauthn';

export const SESSION_PATH = `${INTERNAL}/session`;
export const SECOND_FACTOR_PATH = `${INTERNAL}/session/second-factor`;
export const STATE_PATH = `${INTERNAL}/ui/state`;
export const PASSKEY_OPTIONS_PATH = `${SECOND_FACTOR_PATH}/passkey-options`;
export const SECURITY_PATH = '/user/security';

export type CodeMethod = 'totp' | 'recovery';
export type SecondFactorMethod = CodeMethod | 'passkey';

export interface PasskeyChallenge {
  ceremony: string;
  publicKey: AssertionOptions;
}

export interface SecondFactorAnswer {
  methods?: SecondFactorMethod[];
  expiresAt?: string;
  restricted?: boolean;
}

export interface SignInAnswer {
  authenticated: boolean;
  secondFactor?: SecondFactorAnswer | null;
}

let leaving = false;
let ended = false;

export function signIn(username: string, password: string): Promise<SignInAnswer> {
  return api.post<SignInAnswer>(SESSION_PATH, { username, password }, { quietUnauthorized: true });
}

export function secondStep(method: CodeMethod, code: string): Promise<unknown> {
  return api.post(SECOND_FACTOR_PATH, { method, code }, { quietUnauthorized: true });
}

export function passkeyChallenge(): Promise<PasskeyChallenge> {
  return api.post<PasskeyChallenge>(PASSKEY_OPTIONS_PATH, undefined, { quietUnauthorized: true });
}

export function passkeySecondStep(ceremony: string, credential: AssertionCredential): Promise<unknown> {
  return api.post(
    SECOND_FACTOR_PATH,
    { method: 'passkey', ceremony, credential },
    { quietUnauthorized: true },
  );
}

export function signOut(): Promise<void> {
  return api.delete(SESSION_PATH, { quietUnauthorized: true });
}

export function expectSignOut(): void {
  leaving = true;
}

export function takeExpectedSignOut(): boolean {
  const expected = leaving;
  leaving = false;
  return expected;
}

export function markSessionEnded(): void {
  ended = true;
}

export function takeSessionEnded(): boolean {
  const was = ended;
  ended = false;
  return was;
}
