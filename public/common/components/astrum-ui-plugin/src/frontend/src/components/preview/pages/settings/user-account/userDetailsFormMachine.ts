import { assign } from 'xstate';
import { ENDPOINTS, restClient } from '../../../../../interface/api';
import {
  createFormMachine,
  type FormContext,
  type ValidationErrors,
} from '../../../../../interface/form';
import ValidationUtils from '../../../../../interface/ValidationUtils';
import UserPagesStrings from '../../../constants/pages/user/UserPagesStrings';

export interface UserDetailsData {
  userId: string;
  firstName: string;
  lastName: string;
  email: string;
  external: boolean;
}

export const DEFAULT_USER_DETAILS_DATA: UserDetailsData = {
  userId: '',
  firstName: '',
  lastName: '',
  email: '',
  external: false,
};

interface RestUserAccount {
  userId?: string;
  firstName?: string;
  lastName?: string;
  email?: string;
  external?: boolean;
}

function restToData(rest: RestUserAccount): UserDetailsData {
  return {
    userId: rest.userId ?? '',
    firstName: rest.firstName ?? '',
    lastName: rest.lastName ?? '',
    email: rest.email ?? '',
    external: rest.external ?? false,
  };
}

/**
 * The name and the email are required, and the email must look like one, as on the Classic account page.
 */
export function validateUserDetails(data: UserDetailsData): ValidationErrors {
  const strings = UserPagesStrings.ACCOUNT_FORM_MACHINE;
  const errors: ValidationErrors = {};
  if (!data.firstName.trim()) {
    errors.firstName = strings.FIELD_REQUIRED;
  }
  if (!data.lastName.trim()) {
    errors.lastName = strings.FIELD_REQUIRED;
  }
  if (!data.email.trim()) {
    errors.email = strings.FIELD_REQUIRED;
  }
  else if (!ValidationUtils.isEmail(data.email.trim())) {
    errors.email = strings.EMAIL_INVALID;
  }
  return errors;
}

/**
 * Form of the name and email of the signed-in user. Loads them from the internal UI endpoint and saves them with a
 * PUT to the same endpoint, which only changes the account of the user who is signed in.
 */
export function createUserDetailsFormMachine() {
  return createFormMachine<UserDetailsData>({
    id: 'user-details-form',
    stayEditableAfterSave: true,
    context: {
      data: { ...DEFAULT_USER_DETAILS_DATA },
    },
    actions: {
      validate: assign((ctx: FormContext<UserDetailsData>) => ({
        validationErrors: validateUserDetails(ctx.data),
      })),
    },
    services: {
      load: async () => {
        const rest = await restClient.get<RestUserAccount>(ENDPOINTS.USER_ACCOUNT);
        return { data: restToData(rest) };
      },
      save: async (ctx: FormContext<UserDetailsData>) => {
        const { userId, firstName, lastName, email, external } = ctx.data;
        if (!userId) {
          throw new Error(UserPagesStrings.ACCOUNT_FORM_MACHINE.USER_NOT_FOUND);
        }
        await restClient.put(ENDPOINTS.USER_ACCOUNT, {
          userId,
          firstName: firstName.trim(),
          lastName: lastName.trim(),
          email: email.trim(),
          external,
        });
      },
    },
  });
}
