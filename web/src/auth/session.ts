import { api, INTERNAL } from '../api';

export const SESSION_PATH = `${INTERNAL}/session`;
export const STATE_PATH = `${INTERNAL}/ui/state`;

let leaving = false;
let ended = false;

export function signIn(username: string, password: string): Promise<unknown> {
  return api.post(SESSION_PATH, { username, password }, { quietUnauthorized: true });
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
