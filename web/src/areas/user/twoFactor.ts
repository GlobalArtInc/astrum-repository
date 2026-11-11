import { api, INTERNAL } from '../../api';

export const TWO_FACTOR = `${INTERNAL}/ui/user/two-factor`;
export const TOTP = `${TWO_FACTOR}/totp`;
export const TOTP_CONFIRM = `${TOTP}/confirm`;
export const RECOVERY_CODES = `${TWO_FACTOR}/recovery-codes`;
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

export function qrDataUri(svg: string): string {
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}
