import UserTokensStrings from '../../../../constants/pages/settings/security/UserTokensStrings';


/**
 * User token settings data model
 */
export interface UserTokenSettings {
  enabled: boolean;
  protectContent: boolean;
  expirationEnabled: boolean;
  expirationDays: number;
}

/**
 * Default user token settings
 */
export const DEFAULT_USER_TOKEN_SETTINGS: UserTokenSettings = {
  enabled: false,
  protectContent: false,
  expirationEnabled: false,
  expirationDays: 30,
};

/**
 * Confirmation string required for reset all tokens
 * Must be typed exactly to confirm the reset action
 */
export const RESET_CONFIRMATION_STRING = UserTokensStrings.TYPES.RESET_ALL_TOKENS;
