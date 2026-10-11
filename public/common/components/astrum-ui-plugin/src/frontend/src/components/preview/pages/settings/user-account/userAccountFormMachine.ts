

import { assign } from 'xstate';
import { ENDPOINTS, restClient } from '../../../../../interface/api';
import { ExtJS } from '../../../../../interface/ExtJS';
import {
  createFormMachine,
  type FormContext,
  type ValidationErrors,
} from '../../../../../interface/form';
import UserPagesStrings from '../../../constants/pages/user/UserPagesStrings';

const PASSWORD_MIN_LENGTH = 8;

export interface UserAccountData {
  userId: string;
  firstName: string;
  lastName: string;
  email: string;
  external: boolean;
  currentPassword: string;
  newPassword: string;
  confirmPassword: string;
}

export const DEFAULT_USER_ACCOUNT_DATA: UserAccountData = {
  userId: '',
  firstName: '',
  lastName: '',
  email: '',
  external: false,
  currentPassword: '',
  newPassword: '',
  confirmPassword: '',
};

interface RestUserAccount {
  userId?: string;
  firstName?: string;
  lastName?: string;
  email?: string;
  external?: boolean;
}

function restToData(rest: RestUserAccount): UserAccountData {
  return {
    userId: rest.userId ?? '',
    firstName: rest.firstName ?? '',
    lastName: rest.lastName ?? '',
    email: rest.email ?? '',
    external: rest.external ?? false,
    currentPassword: '',
    newPassword: '',
    confirmPassword: '',
  };
}

/**
 * Validates the password-change portion of the account form.
 *
 * Note: `currentPassword` presence is deliberately NOT enforced here. Emptiness of any
 * single field just means "user hasn't started changing password yet" — surfacing that as
 * a validation error would light up the whole form on load. The "must fill all three
 * fields before submitting" gate lives in `isPasswordFormReady`, which the UI uses to
 * enable/disable the submit button.
 */
export function validateUserAccount(data: UserAccountData): ValidationErrors {
  const errors: ValidationErrors = {};
  const { currentPassword, newPassword, confirmPassword } = data;
  const anyPasswordFieldFilled =
    Boolean(currentPassword) || Boolean(newPassword) || Boolean(confirmPassword);

  if (!anyPasswordFieldFilled) {
    return errors;
  }

  if (newPassword && newPassword.length < PASSWORD_MIN_LENGTH) {
    errors.newPassword = UserPagesStrings.ACCOUNT_FORM_MACHINE.PASSWORD_MUST_LEAST_CHARACTERS(PASSWORD_MIN_LENGTH);
  }

  if (newPassword && currentPassword && newPassword === currentPassword) {
    errors.newPassword = UserPagesStrings.ACCOUNT_FORM_MACHINE.PASSWORD_MUST_DIFFER;
  }

  if (newPassword !== confirmPassword) {
    errors.confirmPassword = UserPagesStrings.ACCOUNT_FORM_MACHINE.PASSWORDS_DO_NOT_MATCH;
  }

  return errors;
}

/**
 * True when every password field is filled and there are no validation errors,
 * meaning the change-password form is ready to submit.
 */
export function isPasswordFormReady(data: UserAccountData): boolean {
  const { currentPassword, newPassword, confirmPassword } = data;
  if (!currentPassword || !newPassword || !confirmPassword) {
    return false;
  }
  return Object.keys(validateUserAccount(data)).length === 0;
}

/**
 * Proves the current password with an authentication ticket, as the Classic account page does, so that a password
 * cannot be changed by anyone who merely finds the session open.
 */
async function authenticationTicket(userId: string, password: string): Promise<string> {
  const response = (await ExtJS.fetchAuthenticationToken(userId, password)) as
    { success?: boolean; data?: string } | null | undefined;
  if (!response?.success || !response.data) {
    throw new Error(UserPagesStrings.ACCOUNT_FORM_MACHINE.CURRENT_PASSWORD_WRONG);
  }
  return response.data;
}

/**
 * User account password form machine.
 *
 * Loads the current user's profile (userId, firstName, lastName, email, external) from
 * the internal UI endpoint; its editable surface is the change-password form (currentPassword,
 * newPassword, confirmPassword). The name and email are edited by the details form.
 *
 * Save proves the current password with an authentication ticket and PUTs the new password
 * with that ticket to /service/rest/internal/ui/user/{userId}/password. Password validation
 * only fires when the user has begun typing in any password field.
 */
export function createUserAccountFormMachine() {
  return createFormMachine<UserAccountData>({
    id: 'user-account-form',
    resetAfterSave: true,
    context: {
      data: { ...DEFAULT_USER_ACCOUNT_DATA },
    },
    actions: {
      validate: assign((ctx: FormContext<UserAccountData>) => ({
        validationErrors: validateUserAccount(ctx.data),
      })),
    },
    services: {
      load: async () => {
        const rest = await restClient.get<RestUserAccount>(ENDPOINTS.USER_ACCOUNT);
        return { data: restToData(rest) };
      },
      save: async (ctx: FormContext<UserAccountData>) => {
        const { userId, currentPassword, newPassword } = ctx.data;
        if (!userId) {
          throw new Error(UserPagesStrings.ACCOUNT_FORM_MACHINE.USER_NOT_FOUND);
        }
        const authToken = await authenticationTicket(userId, currentPassword);
        await restClient.put(
          `${ENDPOINTS.USER_ACCOUNT}/${encodeURIComponent(userId)}/password`,
          { authToken, password: newPassword }
        );
      },
    },
  });
}
