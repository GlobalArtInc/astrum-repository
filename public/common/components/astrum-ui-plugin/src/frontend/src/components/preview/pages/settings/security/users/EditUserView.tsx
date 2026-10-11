

import React, { useCallback } from 'react';
import { Box, Button, Flex, Switch, Text } from '@radix-ui/themes';
import { RefreshCw, Trash2 } from 'lucide-react';

import { PageHeader } from '../../../../shared';
import {
  ConfirmDialog,
  SettingsButton,
  SettingsForm,
} from '../../../../shared/form';
import { UserForm } from './UserForm';
import { useUsersForm } from './useUsersForm';
import { User, DEFAULT_SOURCE, getFullName } from './types';

import './EditUserView.scss';
import UsersStrings from '../../../../constants/pages/settings/security/UsersStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export interface EditUserViewProps {
  /** True for Create route, false for Edit route. */
  isCreate: boolean;
  /** User ID from the route (Edit route only). */
  userId?: string | null;
  /** User source/realm from the route (Edit route only). */
  userSource?: string | null;
  /** Preloaded user (may arrive after the machine's own fetch). */
  user?: User | null;
  /** True when the current admin has astrum:users:delete. */
  canDelete: boolean;
  /** Human-readable reason the current user cannot be deleted, or null. */
  protectionReason: string | null;
  /** Parent handler that opens the delete confirmation modal. */
  onDeleteRequest: () => void;
  /** Parent handler for post-save side effects (list refresh). */
  onSuccess: () => void;
  /** Parent handler for navigation back to the list. */
  onCancel: () => void;
  /** True while a delete request is in flight (disables the toolbar Delete). */
  isDeleting?: boolean;
}

const navigateToSettings = () => {
  window.location.hash = '#preview/admin/settings';
};

export function EditUserView({
  isCreate,
  userId,
  userSource,
  user,
  canDelete,
  protectionReason,
  onDeleteRequest,
  onSuccess,
  onCancel,
  isDeleting = false,
}: EditUserViewProps) {
  const vm = useUsersForm({
    userId: isCreate ? undefined : userId ?? undefined,
    userSource: userSource ?? DEFAULT_SOURCE,
    user: user ?? undefined,
    onSave: async () => {
      onSuccess();
    },
    onCancel,
  });

  const { formData, currentUser, isExternal, showsUserTokenReset, form } = vm;
  const statusToggleDisabled =
    !currentUser || isExternal || vm.isSaving || vm.isTogglingStatus;

  const title = isCreate
    ? UsersStrings.EDIT.CREATE_USER
    : currentUser
      ? UsersStrings.EDIT.EDIT(getFullName(currentUser) || currentUser.userId)
      : UsersStrings.EDIT.EDIT_USER;

  const description = isCreate
    ? UsersStrings.EDIT.ADD_NEW_LOCAL_USER_ACCOUNT
    : UsersStrings.EDIT.UPDATE_USER_PROFILE_ROLES_EFFECTIVE;

  const statusLabel = formData.status ? UsersStrings.EDIT.ACTIVE : UsersStrings.EDIT.INACTIVE;
  const isProtectedForDelete = protectionReason !== null;

  const headerActions = !isCreate ? (
    <Flex align="center" gap="3" data-testid="edit-user-toolbar">
      <Flex align="center" gap="2">
        <Switch
          checked={!!formData.status}
          onCheckedChange={vm.toggleStatus}
          disabled={statusToggleDisabled}
          aria-label={UsersStrings.EDIT.TOGGLE_USER_STATUS_CURRENTLY(statusLabel)}
          data-testid="user-status-toggle"
          data-analytics-id="as-user-toggle-status"
        />
        <Text size="2" data-testid="user-status-label">
          {statusLabel}
        </Text>
      </Flex>

      {showsUserTokenReset && (
        <SettingsButton
          variant="secondary"
          icon={RefreshCw}
          onClick={vm.openResetTokenDialog}
          loading={vm.isResettingToken}
          testId="reset-user-token-button"
          data-analytics-id="as-user-reset-token"
        >
          {UsersStrings.EDIT.RESET_USER_TOKEN}
        </SettingsButton>
      )}
    </Flex>
  ) : undefined;

  const deleteFooter =
    !isCreate && canDelete && currentUser ? (
      <Button
        type="button"
        data-testid="page-delete-user"
        variant="soft"
        color="red"
        onClick={onDeleteRequest}
        disabled={isProtectedForDelete || isDeleting}
        title={protectionReason ?? undefined}
        aria-label={
          isProtectedForDelete
            ? UsersStrings.EDIT.DELETE_USER(protectionReason)
            : UsersStrings.EDIT.DELETE_USER_TITLE
        }
        data-analytics-id="as-user-delete"
      >
        <Trash2 size={16} />
        {UsersStrings.EDIT.DELETE_USER_TITLE}
      </Button>
    ) : undefined;

  const handleDiscardConfirm = useCallback(() => {
    form.reset();
    onCancel();
  }, [form, onCancel]);

  return (
    <Box className="edit-user-view">
      <PageHeader
        title={title}
        description={description}
        actions={headerActions}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
          { label: UsersStrings.EDIT.USERS, onClick: onCancel },
          {
            label: isCreate
              ? CommonStrings.ACTIONS.CREATE
              : currentUser?.userId || userId || CommonStrings.MESSAGES.LOADING,
          },
        ]}
      />

      <SettingsForm
        testId="user-form"
        onSubmit={vm.submit}
        onCancel={vm.cancel}
        pristine={form.isPristine}
        loading={vm.isSaving}
        error={vm.saveError || undefined}
        submitLabel={isCreate ? CommonStrings.ACTIONS.CREATE : CommonStrings.ACTIONS.SAVE}
        externalDirtyTracking={true}
        onDiscardConfirm={handleDiscardConfirm}
        footerExtra={deleteFooter}
      >
        <UserForm vm={vm} />
      </SettingsForm>

      <ConfirmDialog
        open={vm.resetTokenDialogOpen}
        testId="reset-user-token-dialog"
        onOpenChange={(open) => {
          if (!open) vm.closeResetTokenDialog();
        }}
        title={UsersStrings.EDIT.RESET_USER_TOKEN}
        message={UsersStrings.EDIT.RESET_TOKEN_CONFIRM(currentUser ? getFullName(currentUser) || currentUser.userId : '')}
        confirmLabel={UsersStrings.EDIT.RESET}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="danger"
        onConfirm={vm.confirmResetToken}
      />
    </Box>
  );
}

export default EditUserView;
