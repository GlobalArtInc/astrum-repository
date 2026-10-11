

import { useCallback, useMemo, useRef, useState } from 'react';
import { ExtJS } from '@globalart/astrum-ui-plugin';
import { useForm } from '../../../../../../interface/form';
import type { ValidationErrors } from '../../../../../../interface/form';
import { useToast } from '../../../../shared';
import { useUsersApi } from './useUsersApi';
import {
  createUsersFormMachine,
  RoleRef,
  UserSourceRef,
} from './usersFormMachine';
import {
  User,
  UserFormData,
  DEFAULT_SOURCE,
  isExternalUser,
  getFullName,
} from './types';
import UsersStrings from '../../../../constants/pages/settings/security/UsersStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export interface UseUsersFormOptions {
  userId?: string;
  userSource?: string;
  user?: User;
  onSave?: (data: UserFormData) => Promise<void>;
  onCancel: () => void;
  createUser?: (data: UserFormData) => Promise<User | void>;
  updateUser?: (
    userId: string,
    data: UserFormData,
    source: string,
  ) => Promise<User | void>;
  changePassword?: (userId: string, newPassword: string) => Promise<void>;
  resetUserToken?: (userId: string, realm: string) => Promise<void>;
  patchUserStatus?: (currentUser: User, active: boolean) => Promise<User | void>;
  onStatusChanged?: (active: boolean) => void;
}

export interface UseUsersFormResult {
  form: ReturnType<typeof useForm>;
  user: User | null;
  isCreate: boolean;
  currentUser: User | null;

  formData: UserFormData;
  allRoles: RoleRef[];
  sources: UserSourceRef[];
  externalRoles: string[];

  isExternal: boolean;
  isDirty: boolean;
  rolesDirty: boolean;
  isPro: boolean;
  isUserTokenCapabilityActive: boolean;
  canResetUserToken: boolean;
  showsUserTokenReset: boolean;
  isLoading: boolean;
  isSaving: boolean;
  saveError: string | undefined;
  validationErrors: ValidationErrors;

  showPasswordChange: boolean;
  resetTokenDialogOpen: boolean;
  isResettingToken: boolean;
  isTogglingStatus: boolean;

  submit(): void;
  cancel(): void;
  setRoles(roleIds: string[]): void;
  showPasswordChangeSection(): void;
  hidePasswordChangeSection(): void;
  resetPasswordFields(): void;
  openResetTokenDialog(): void;
  closeResetTokenDialog(): void;
  confirmResetToken(): Promise<void>;
  toggleStatus(active: boolean): Promise<void>;
}

/**
 * ViewModel for the Users form. Owns machine wiring, API composition,
 * interaction state, and semantic commands. The View consumes the returned
 * projections and commands only; it must not reach into machine context,
 * ExtJS, or useUsersApi directly.
 */
export function useUsersForm(
  options: UseUsersFormOptions,
): UseUsersFormResult {
  const {
    userId,
    userSource = DEFAULT_SOURCE,
    user,
    onSave,
    onCancel,
  } = options;

  const toast = useToast();
  const api = useUsersApi();
  const createUser = options.createUser ?? api.createUser;
  const updateUser = options.updateUser ?? api.updateUser;
  const changePassword = options.changePassword ?? api.changePassword;
  const resetUserToken = options.resetUserToken ?? api.resetUserToken;
  const patchUserStatus = options.patchUserStatus ?? api.patchUserStatus;
  const onStatusChanged = options.onStatusChanged;

  const isCreate = !userId && !user;

  const [showPasswordChange, setShowPasswordChange] = useState(false);
  const [resetTokenDialogOpen, setResetTokenDialogOpen] = useState(false);
  const [isResettingToken, setIsResettingToken] = useState(false);
  const [isTogglingStatus, setIsTogglingStatus] = useState(false);
  // Synchronous guard against back-to-back clicks: setState updates are batched, so
  // a second toggleStatus fired before React re-renders would still see isTogglingStatus=false.
  const togglingRef = useRef(false);

  // Machine is stable for the lifetime of a given route. `user` is intentionally
  // NOT in the deps: UsersPage's fetch may resolve after the machine has already
  // loaded via its own `load` service, and a mid-edit `user`-prop change would
  // recreate the machine and wipe typed input. Refresh is driven by
  // `key={refreshKey}` on EditUserView (in UsersPage), which force-remounts the
  // whole subtree and re-runs this memo cleanly.
  const machine = useMemo(
    () => createUsersFormMachine(userId, userSource, user),
    // eslint-disable-next-line react-hooks/exhaustive-deps -- see comment above
    [userId, userSource],
  );

  const form = useForm(machine, {
    actions: {
      onCancel: onCancel,
    },
    services: {
      save: async (ctx: { data: UserFormData; user: User | null }) => {
        try {
          const userToUpdate = user || ctx.user;

          if (isCreate) {
            await createUser(ctx.data);
            toast.success(UsersStrings.MESSAGES.USER_CREATED_SUCCESSFULLY(ctx.data.userId));
          } else if (userToUpdate) {
            const source = userToUpdate.source || userSource;
            await updateUser(userToUpdate.userId, ctx.data, source);
            if (ctx.data.password) {
              await changePassword(userToUpdate.userId, ctx.data.password);
            }
            toast.success(UsersStrings.MESSAGES.USER_UPDATED_SUCCESSFULLY(ctx.data.userId));
          } else if (ctx.data?.userId) {
            const source = ctx.data.source || userSource;
            await updateUser(ctx.data.userId, ctx.data, source);
            if (ctx.data.password) {
              await changePassword(ctx.data.userId, ctx.data.password);
            }
            toast.success(UsersStrings.MESSAGES.USER_UPDATED_SUCCESSFULLY(ctx.data.userId));
          } else {
            throw new Error(
              UsersStrings.MESSAGES.USER_DATA_NOT_LOADED_PLEASE,
            );
          }

          if (onSave) {
            await onSave(ctx.data);
          }
          onCancel();
        } catch (err) {
          toast.error(err instanceof Error ? err.message : CommonStrings.MESSAGES.OPERATION_FAILED);
          throw err;
        }
      },
    },
  });

  const rawContext = (form.state as {
    context: {
      user: User | null;
      allRoles?: RoleRef[];
      userSources?: UserSourceRef[];
      initialRoles?: string[];
    };
  }).context;
  const loadedUser = rawContext.user;
  const currentUser: User | null = user ?? loadedUser;

  const formData = (form.data ?? {}) as UserFormData;
  const allRoles = rawContext.allRoles ?? [];
  const sources = rawContext.userSources ?? [];
  const externalRoles = currentUser?.externalRoles ?? [];
  const initialRolesFromMachine = rawContext.initialRoles;

  const isPro = ExtJS.isProEdition();
  const extState = ExtJS.state?.();
  const activeCapabilities: string[] =
    extState?.getValue?.('capabilityActiveTypes') ?? [];
  const isUserTokenCapabilityActive = activeCapabilities.includes('usertoken');
  const canResetUserToken = ExtJS.checkPermission(
    'astrum:usertoken-user:delete',
  );

  const isExternal = currentUser
    ? isExternalUser(currentUser.source)
    : isExternalUser(formData.source ?? DEFAULT_SOURCE);

  const isDirty = !form.isPristine;
  const isLoading = form.isLoading;
  const isSaving = form.isSaving;
  const saveError = form.saveError ?? undefined;
  const validationErrors: ValidationErrors = form.validationErrors ?? {};

  // Prefer the machine-context snapshot (frozen at load) so rolesDirty is
  // stable against later mutations of the currentUser prop reference.
  const initialRoleIds = useMemo(
    () =>
      [...(initialRolesFromMachine ?? currentUser?.roles ?? [])].sort(),
    [initialRolesFromMachine, currentUser?.roles],
  );
  const rolesDirty = useMemo(() => {
    const pending = [...(formData.roles ?? [])].sort();
    if (pending.length !== initialRoleIds.length) return true;
    for (let i = 0; i < pending.length; i += 1) {
      if (pending[i] !== initialRoleIds[i]) return true;
    }
    return false;
  }, [formData.roles, initialRoleIds]);

  const showsUserTokenReset =
    !isCreate &&
    !isExternal &&
    isPro &&
    isUserTokenCapabilityActive &&
    canResetUserToken;

  const submit = useCallback(() => {
    form.submit();
  }, [form]);

  const cancel = useCallback(() => {
    onCancel();
  }, [onCancel]);

  const setRoles = useCallback(
    (roleIds: string[]) => {
      form.send({ type: 'UPDATE', name: 'roles', value: roleIds } as any);
    },
    [form],
  );

  const showPasswordChangeSection = useCallback(() => {
    setShowPasswordChange(true);
  }, []);

  const hidePasswordChangeSection = useCallback(() => {
    setShowPasswordChange(false);
  }, []);

  const resetPasswordFields = useCallback(() => {
    form.send({ type: 'UPDATE', name: 'password', value: '' } as any);
    form.send({ type: 'UPDATE', name: 'passwordConfirm', value: '' } as any);
    setShowPasswordChange(false);
  }, [form]);

  const openResetTokenDialog = useCallback(() => {
    setResetTokenDialogOpen(true);
  }, []);

  const closeResetTokenDialog = useCallback(() => {
    setResetTokenDialogOpen(false);
  }, []);

  const toggleStatus = useCallback(
    async (active: boolean) => {
      if (!currentUser) return;
      if (togglingRef.current) return;
      togglingRef.current = true;
      setIsTogglingStatus(true);
      try {
        await patchUserStatus(currentUser, active);
        form.send({
          type: 'SYNC_FIELD',
          name: 'status',
          value: active,
        } as any);
        toast.success(
          active
            ? UsersStrings.MESSAGES.USER_ACTIVATED(currentUser.userId)
            : UsersStrings.MESSAGES.USER_DEACTIVATED(currentUser.userId),
        );
        onStatusChanged?.(active);
      } catch (err) {
        toast.error(
          err instanceof Error ? err.message : UsersStrings.MESSAGES.STATUS_UPDATE_FAILED,
        );
      } finally {
        togglingRef.current = false;
        setIsTogglingStatus(false);
      }
    },
    [currentUser, patchUserStatus, form, toast, onStatusChanged],
  );

  const confirmResetToken = useCallback(async () => {
    if (!currentUser) return;
    setResetTokenDialogOpen(false);
    setIsResettingToken(true);
    try {
      await resetUserToken(
        currentUser.userId,
        currentUser.realm || currentUser.source,
      );
      toast.success(
        UsersStrings.MESSAGES.USER_TOKEN_HAS_BEEN_RESET(getFullName(currentUser)),
      );
    } catch (err) {
      toast.error(err instanceof Error ? err.message : UsersStrings.MESSAGES.USER_TOKEN_RESET_FAILED);
    } finally {
      setIsResettingToken(false);
    }
  }, [currentUser, resetUserToken, toast]);

  return {
    form,
    user: loadedUser,
    isCreate,
    currentUser,
    formData,
    allRoles,
    sources,
    externalRoles,
    isExternal,
    isDirty,
    rolesDirty,
    isPro,
    isUserTokenCapabilityActive,
    canResetUserToken,
    showsUserTokenReset,
    isLoading,
    isSaving,
    saveError,
    validationErrors,
    showPasswordChange,
    resetTokenDialogOpen,
    isResettingToken,
    isTogglingStatus,
    submit,
    cancel,
    setRoles,
    showPasswordChangeSection,
    hidePasswordChangeSection,
    resetPasswordFields,
    openResetTokenDialog,
    closeResetTokenDialog,
    confirmResetToken,
    toggleStatus,
  };
}
