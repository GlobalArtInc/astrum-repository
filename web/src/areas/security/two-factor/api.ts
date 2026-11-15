import { path } from '../../../api';
import { SECURITY, type User } from '../api';

export const TWO_FACTOR = `${SECURITY}/two-factor`;

export type SecondFactorRequirement = 'policy' | 'required' | 'exempt';

export interface UserSecondFactor {
  enrolled: boolean;
  required: boolean;
  requirement: SecondFactorRequirement;
}

export interface UserWithSecondFactor extends User {
  twoFactor?: UserSecondFactor;
}

export interface UserSecondFactorStatus {
  userId: string;
  enrolled: boolean;
  totp: boolean;
  recoveryCodesLeft: number;
  requirement: SecondFactorRequirement;
  requirementSource: SecondFactorRequirement;
  required: boolean;
  enrollBy: string | null;
  lockedUntil: string | null;
}

export interface RequirementBody {
  requirement: SecondFactorRequirement;
}

export interface RequiredFor {
  everyone: boolean;
  administrators: boolean;
  roles: string[];
}

export interface RelyingPartyPolicy {
  id: string | null;
  origins: string[];
}

export interface TwoFactorPolicy {
  totp: boolean;
  passkeys: boolean;
  required: RequiredFor;
  graceDays: number;
  requiredSince: string | null;
  relyingParty: RelyingPartyPolicy;
}

export function userTwoFactorPath(userId: string): string {
  return `${SECURITY}${path`/users/${userId}/two-factor`}`;
}

export function userRequirementPath(userId: string): string {
  return `${SECURITY}${path`/users/${userId}/requirement`}`;
}
