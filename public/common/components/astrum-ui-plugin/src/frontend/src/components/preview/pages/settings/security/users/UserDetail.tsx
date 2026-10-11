

import React, { useState } from 'react';
import { Box, Flex, Text, Separator } from '@radix-ui/themes';
import { Loader2, Key, RefreshCw, AlertTriangle } from 'lucide-react';
import { ExtJS } from '@globalart/astrum-ui-plugin';

import { MetadataGrid, StatusBadge, type StatusType } from '../../../../shared';

import {
  SettingsFormSection,
  SettingsButton,
  SettingsAlert,
  SettingsPasswordInput,
  ConfirmDialog,
} from '../../../../shared/form';
import { useToast } from '../../../../shared';
import { useUsersApi } from './useUsersApi';
import {
  User,
  isExternalUser,
  getFullName,
} from './types';

import './UserDetail.scss';
import UsersStrings from '../../../../constants/pages/settings/security/UsersStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface UserDetailProps {
  user: User | null;
  loading: boolean;
  canEdit: boolean;
  onCancel: () => void;
}

/**
 * UserDetail - Detailed view and edit form for a single user
 */
export function UserDetail({
  user,
  loading,
  canEdit,
  onCancel,
}: UserDetailProps) {
  const { changePassword, resetUserToken, loading: apiLoading, error: apiError, setError } = useUsersApi();
  const toast = useToast();
  
  const [showChangePassword, setShowChangePassword] = useState(false);
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [resetTokenDialogOpen, setResetTokenDialogOpen] = useState(false);

  const isPro = ExtJS.isProEdition();
  const isAdmin = ExtJS.checkPermission('astrum:*');
  const state = ExtJS.state();
  const isAnonymous = user?.userId === state?.getValue?.('anonymousUsername');
  const isExternal = user ? isExternalUser(user.source) : false;
  const activeCapabilities = state?.getValue?.('capabilityActiveTypes') || [];
  const isUserTokenCapabilityActive = activeCapabilities.includes('usertoken');
  const canResetUserToken = ExtJS.checkPermission('astrum:usertoken-user:delete');

  const showChangePasswordButton = canEdit && !isExternal && !isAnonymous && isAdmin;
  const showResetTokenButton = isPro && !isExternal && isUserTokenCapabilityActive && canResetUserToken;

  // Loading state
  if (loading) {
    return (
      <Flex
        align="center"
        justify="center"
        className="user-detail__loading"
        role="status"
        aria-live="polite"
        aria-busy="true"
      >
        <Loader2 size={24} className="user-detail__spinner" aria-hidden="true" />
        <Text size="2">{UsersStrings.DETAIL.LOADING_USER_DETAILS}</Text>
      </Flex>
    );
  }

  // No user found
  if (!user) {
    return (
      <Box
        className="user-detail__not-found"
        role="alert"
        aria-live="assertive"
      >
        <AlertTriangle size={24} aria-hidden="true" />
        <Text size="2">{UsersStrings.DETAIL.USER_NOT_FOUND}</Text>
      </Box>
    );
  }

  const handleChangePassword = async () => {
    setPasswordError(null);

    if (!newPassword) {
      setPasswordError(UsersStrings.DETAIL.NEW_PASSWORD_REQUIRED);
      return;
    }
    if (newPassword !== confirmPassword) {
      setPasswordError(UsersStrings.DETAIL.PASSWORDS_DO_NOT_MATCH);
      return;
    }

    try {
      await changePassword(user.userId, newPassword);
      toast.success(UsersStrings.DETAIL.PASSWORD_CHANGED_SUCCESSFULLY(getFullName(user)));
      setShowChangePassword(false);
      setNewPassword('');
      setConfirmPassword('');
    } catch (_err) {
      // Error is set by the API hook
    }
  };

  const handleResetTokenClick = () => {
    setResetTokenDialogOpen(true);
  };

  const handleResetTokenConfirm = async () => {
    setResetTokenDialogOpen(false);
    try {
      await resetUserToken(user.userId, user.realm || user.source);
      toast.success(UsersStrings.DETAIL.USER_TOKEN_HAS_BEEN_RESET(getFullName(user)));
    } catch (_err) {
      // Error is set by the API hook
    }
  };

  // Read-only view for users without edit permission
  if (!canEdit) {
    const userStatusMap: Record<string, StatusType> = {
      active: 'success',
      disabled: 'offline',
    };
    const statusType: StatusType = userStatusMap[user.status] ?? 'unknown';
    const statusLabel = user.status.charAt(0).toUpperCase() + user.status.slice(1);

    return (
      <Box className="user-detail">
        <SettingsFormSection title={UsersStrings.DETAIL.USER_INFORMATION} defaultOpen>
          <MetadataGrid items={[
            { label: 'ID', value: user.userId },
            { label: UsersStrings.DETAIL.FIRST_NAME, value: user.firstName },
            { label: UsersStrings.DETAIL.LAST_NAME, value: user.lastName },
            { label: UsersStrings.DETAIL.EMAIL, value: user.emailAddress || user.email },
            { label: CommonStrings.LABELS.STATUS, value: <StatusBadge status={statusType} label={statusLabel} size="small" /> },
            { label: UsersStrings.DETAIL.SOURCE, value: isExternal ? user.source : UsersStrings.DETAIL.LOCAL },
          ]} />
        </SettingsFormSection>

        <SettingsFormSection title={UsersStrings.DETAIL.ASSIGNED_ROLES}>
          <Box className="user-detail__roles">
            {user.roles && user.roles.length > 0 ? (
              user.roles.map((role) => (
                <Text key={role} size="2" className="user-detail__role">
                  {role}
                </Text>
              ))
            ) : (
              <Text size="2" className="user-detail__no-roles">
                {UsersStrings.DETAIL.NO_ROLES_ASSIGNED}
              </Text>
            )}
          </Box>
        </SettingsFormSection>

        <Flex gap="3" className="user-detail__actions">
          <SettingsButton variant="secondary" onClick={onCancel}>
            {CommonStrings.ACTIONS.BACK_TO_LIST}
          </SettingsButton>
        </Flex>
      </Box>
    );
  }

  // Edit view
  return (
    <Box className="user-detail">
      {/* Error Messages */}
      {apiError && (
        <SettingsAlert type="error" onClose={() => setError(null)}>
          {apiError}
        </SettingsAlert>
      )}

      {/* Account Actions -- grouped in one section */}
      {(showChangePasswordButton || showResetTokenButton) && (
        <SettingsFormSection title={UsersStrings.DETAIL.ACCOUNT_ACTIONS}>
          <Flex direction="column" gap="3">
            {showChangePasswordButton && (
              <>
                <Flex align="center" justify="between" className="user-detail__action-row">
                  <Box>
                    <Text size="2" weight="medium">{UsersStrings.DETAIL.CHANGE_PASSWORD}</Text>
                    <Text as="p" size="1" color="gray">{UsersStrings.DETAIL.SET_NEW_PASSWORD_USER}</Text>
                  </Box>
                  {!showChangePassword && (
                    <SettingsButton variant="secondary" icon={Key} onClick={() => setShowChangePassword(true)}>
                      {UsersStrings.DETAIL.CHANGE_PASSWORD}
                    </SettingsButton>
                  )}
                </Flex>

                {showChangePassword && (
                  <Box className="user-detail__password-form">
                    {passwordError && (
                      <SettingsAlert type="error" onClose={() => setPasswordError(null)}>
                        {passwordError}
                      </SettingsAlert>
                    )}
                    <SettingsPasswordInput
                      name="newPassword"
                      label={UsersStrings.DETAIL.NEW_PASSWORD}
                      value={newPassword}
                      onChange={setNewPassword}
                      required
                      autoComplete="new-password"
                    />
                    <SettingsPasswordInput
                      name="confirmPassword"
                      label={UsersStrings.DETAIL.CONFIRM_NEW_PASSWORD}
                      value={confirmPassword}
                      onChange={setConfirmPassword}
                      required
                      autoComplete="new-password"
                    />
                    <Flex gap="2">
                      <SettingsButton
                        variant="primary"
                        onClick={handleChangePassword}
                        loading={apiLoading}
                      >
                        {UsersStrings.DETAIL.CHANGE_PASSWORD}
                      </SettingsButton>
                      <SettingsButton
                        variant="secondary"
                        onClick={() => {
                          setShowChangePassword(false);
                          setNewPassword('');
                          setConfirmPassword('');
                          setPasswordError(null);
                        }}
                      >
                        {CommonStrings.ACTIONS.CANCEL}
                      </SettingsButton>
                    </Flex>
                  </Box>
                )}
              </>
            )}

            {showChangePasswordButton && showResetTokenButton && <Separator size="4" />}

            {showResetTokenButton && (
              <Flex align="center" justify="between" className="user-detail__action-row">
                <Box>
                  <Text size="2" weight="medium">{UsersStrings.DETAIL.RESET_USER_TOKEN}</Text>
                  <Text as="p" size="1" color="gray">{UsersStrings.DETAIL.INVALIDATE_CURRENT_TOKEN_FORCE_REGENERATION}</Text>
                </Box>
                <SettingsButton
                  variant="secondary"
                  icon={RefreshCw}
                  onClick={handleResetTokenClick}
                  loading={apiLoading}
                >
                  {UsersStrings.DETAIL.RESET_TOKEN}
                </SettingsButton>
              </Flex>
            )}
          </Flex>
        </SettingsFormSection>
      )}

      {/* Reset Token Confirmation Dialog */}
      <ConfirmDialog
        open={resetTokenDialogOpen}
        testId="reset-user-token-dialog"
        onOpenChange={setResetTokenDialogOpen}
        title={UsersStrings.DETAIL.RESET_USER_TOKEN}
        message={UsersStrings.DETAIL.RESET_TOKEN_CONFIRM(user ? getFullName(user) : '')}
        confirmLabel={UsersStrings.DETAIL.RESET}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="danger"
        onConfirm={handleResetTokenConfirm}
      />
    </Box>
  );
}

export default UserDetail;

