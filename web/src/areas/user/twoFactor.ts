import { ApiError, api, INTERNAL, request } from '../../api';
import type { RegistrationCredential, RegistrationOptions } from '../../auth/webauthn';

export const TWO_FACTOR = `${INTERNAL}/ui/user/two-factor`;
export const TOTP = `${TWO_FACTOR}/totp`;
export const TOTP_CONFIRM = `${TOTP}/confirm`;
export const RECOVERY_CODES = `${TWO_FACTOR}/recovery-codes`;
export const PASSKEYS = `${TWO_FACTOR}/passkeys`;
export const PASSKEY_OPTIONS = `${PASSKEYS}/options`;
export const LOW_RECOVERY_CODES = 2;

export interface TwoFactorStatus {
  enrolled: boolean;
  totp: boolean;
  required: boolean;
  requirementSource: string;
  enrollBy: string | null;
  restricted: boolean;
  recoveryCodesLeft: number;
  lockedUntil: string | null;
  accessTokens: number;
  methods: string[];
  passkeys: Passkey[];
  passkeysAvailable: boolean;
}

export interface Passkey {
  id: string;
  name: string;
  createdAt: string;
  lastUsedAt: string | null;
  backedUp: boolean;
  usable: boolean;
}

export interface PasskeySetup {
  ceremony: string;
  publicKey: RegistrationOptions;
  expiresAt: string;
}

export interface PasskeyBeginBody {
  currentPassword: string;
  name: string;
}

export interface PasskeyRegisterBody {
  ceremony: string;
  credential: RegistrationCredential;
  currentPassword: string;
}

export interface PasskeyAdded {
  passkey: Passkey;
  recoveryCodes?: string[];
}

export interface TotpEnrollment {
  ceremony: string;
  secret: string;
  otpauthUri: string;
  qrSvg: string;
  expiresAt: string;
}

export interface TotpConfirmation {
  recoveryCodes?: string[];
}

export interface RecoveryCodes {
  recoveryCodes: string[];
}

export interface ConfirmBody {
  ceremony: string;
  code: string;
  currentPassword: string;
}

export function beginTotp(currentPassword: string): Promise<TotpEnrollment> {
  return api.post<TotpEnrollment>(TOTP, { currentPassword });
}

export function confirmTotp(body: ConfirmBody): Promise<TotpConfirmation> {
  return api.post<TotpConfirmation>(TOTP_CONFIRM, body);
}

export function removeTotp(currentPassword: string): Promise<void> {
  return api.delete(TOTP, { body: { currentPassword } });
}

export function regenerateRecoveryCodes(currentPassword: string): Promise<RecoveryCodes> {
  return api.post<RecoveryCodes>(RECOVERY_CODES, { currentPassword });
}

export function beginPasskey(body: PasskeyBeginBody): Promise<PasskeySetup> {
  return api.post<PasskeySetup>(PASSKEY_OPTIONS, body);
}

export function registerPasskey(body: PasskeyRegisterBody): Promise<PasskeyAdded> {
  return api.post<PasskeyAdded>(PASSKEYS, body);
}

export function renamePasskey(id: string, name: string): Promise<void> {
  return request<void>(`${PASSKEYS}/${encodeURIComponent(id)}`, { method: 'PATCH', body: { name } });
}

export function removePasskey(id: string, currentPassword: string): Promise<void> {
  return api.delete(`${PASSKEYS}/${encodeURIComponent(id)}`, { body: { currentPassword } });
}

export function fieldMessage(error: unknown, field: string): string | null {
  if (!(error instanceof ApiError)) {
    return null;
  }
  return error.fieldErrors.find((problem) => problem.id === field)?.message ?? null;
}

export function qrDataUri(svg: string): string {
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}
