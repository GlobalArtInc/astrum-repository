import { useEffect, useMemo, useRef } from 'react';
import { useForm } from '../../../../../interface/form';
import { useToast } from '../../../shared';
import { createUserDetailsFormMachine, type UserDetailsData } from './userDetailsFormMachine';
import UserPagesStrings from '../../../constants/pages/user/UserPagesStrings';

/**
 * The form of the name and email of the signed-in user, with a toast when they are saved or saving fails.
 */
export function useUserDetailsForm() {
  const toast = useToast();
  const toastRef = useRef(toast);
  toastRef.current = toast;

  const machine = useMemo(() => createUserDetailsFormMachine(), []);
  const form = useForm<UserDetailsData>(machine);

  const { isSaving, saveError } = form;

  const wasSavingRef = useRef(false);
  useEffect(() => {
    if (wasSavingRef.current && !isSaving) {
      if (saveError) {
        toastRef.current.error(saveError);
      } else {
        toastRef.current.success(UserPagesStrings.ACCOUNT_FORM_HOOK.DETAILS_SAVED);
      }
    }
    wasSavingRef.current = isSaving;
  }, [isSaving, saveError]);

  return form;
}
