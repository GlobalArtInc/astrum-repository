

import { useEffect, useMemo, useRef } from 'react';
import { useForm } from '../../../../../interface/form';
import { useToast } from '../../../shared';
import {
  createUserAccountFormMachine,
  isPasswordFormReady,
  type UserAccountData,
} from './userAccountFormMachine';
import UserPagesStrings from '../../../constants/pages/user/UserPagesStrings';

/**
 * Custom hook for managing the User Account settings form.
 *
 * Wraps createUserAccountFormMachine with toast notifications for load/save success/failure.
 * The account profile fields (userId, firstName, lastName, email, external) are loaded
 * once from the internal UI endpoint; the change-password form is the only editable
 * surface. The machine owns the save PUT request; this hook only wires side-effects
 * (toasts) via state-change effects.
 */
export function useUserAccountForm() {
  const toast = useToast();
  const toastRef = useRef(toast);
  toastRef.current = toast;

  const machine = useMemo(() => createUserAccountFormMachine(), []);
  const form = useForm<UserAccountData>(machine);

  const { hasLoadError, isSaving, saveError } = form;

  useEffect(() => {
    if (hasLoadError) {
      toastRef.current.error(UserPagesStrings.ACCOUNT_FORM_HOOK.FAILED_LOAD_ACCOUNT_INFORMATION);
    }
  }, [hasLoadError]);

  const wasSavingRef = useRef(false);
  useEffect(() => {
    if (wasSavingRef.current && !isSaving) {
      if (saveError) {
        toastRef.current.error(saveError);
      } else {
        toastRef.current.success(UserPagesStrings.ACCOUNT_FORM_HOOK.PASSWORD_CHANGED_SUCCESSFULLY);
      }
    }
    wasSavingRef.current = isSaving;
  }, [isSaving, saveError]);

  return {
    ...form,
    canSubmitPassword: isPasswordFormReady(form.data),
  };
}
