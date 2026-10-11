

import React, { useCallback, useMemo } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2, Key, Info } from 'lucide-react';

import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsPasswordInput,
  SettingsButton,
  SettingsTransferList,
} from '../../../../shared/form';
import { RoleExplorerTree } from '../roles/RoleExplorerTree';
import type { UseUsersFormResult } from './useUsersForm';
import { useUserTreePreview } from './useUserTreePreview';
import { getSourceLabel } from './types';

import './UserForm.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import UsersStrings from '../../../../constants/pages/settings/security/UsersStrings';
import plural from '../../../../../../interface/plural';

export interface UserFormProps {
  vm: UseUsersFormResult;
}

/**
 * Body of the Edit/Create User form. Renders Details, Roles, and the live
 * Privileges/Security Tree preview. Meant to be embedded inside a parent
 * SettingsForm; the parent owns title, actions, and the sticky action bar.
 */
export function UserForm({ vm }: UserFormProps) {
  const {
    form,
    formData,
    isCreate,
    isExternal,
    isLoading,
    showPasswordChange,
    currentUser,
    externalRoles,
    allRoles,
    rolesDirty,
    setRoles,
    showPasswordChangeSection,
    resetPasswordFields,
  } = vm;

  const {
    tree: previewTree,
    loading: previewLoading,
    toggleExpand: previewToggleExpand,
    expandAll: previewExpandAll,
    collapseAll: previewCollapseAll,
    setSearchTerm: setPreviewSearchTerm,
  } = useUserTreePreview(formData.roles ?? [], currentUser);

  const availableRoles = useMemo(
    () => allRoles.map((r) => ({ id: r.id, name: r.name })),
    [allRoles],
  );

  const selectedRoles = useMemo(
    () =>
      allRoles
        .filter((r) => (formData.roles ?? []).includes(r.id))
        .map((r) => ({ id: r.id, name: r.name })),
    [allRoles, formData.roles],
  );

  const handleRolesChange = useCallback(
    (next: Array<{ id: string; name: string }>) => {
      setRoles(next.map((r) => r.id));
    },
    [setRoles],
  );

  if (isLoading) {
    return (
      <Flex
        align="center"
        justify="center"
        className="user-form__loading"
        role="status"
        aria-live="polite"
        aria-busy="true"
      >
        <Loader2 size={24} className="user-form__spinner" aria-hidden="true" />
        <Text size="2">{CommonStrings.MESSAGES.LOADING_FORM}</Text>
      </Flex>
    );
  }

  const externalLabel = currentUser
    ? getSourceLabel(currentUser.source)
    : undefined;

  return (
    <Box className="user-form">
      {isExternal && (
        <Box
          className="user-form__external-indicator"
          role="note"
          data-testid="user-form-external-indicator"
        >
          <Info size={16} aria-hidden="true" />
          <Text size="2">
            {externalLabel
              ? UsersStrings.FORM.USER_EXTERNALLY_MANAGED_VIA_PROFILE(externalLabel)
              : UsersStrings.FORM.USER_EXTERNALLY_MANAGED_PROFILE_FIELDS}
          </Text>
        </Box>
      )}

      <SettingsFormSection title={UsersStrings.FORM.DETAILS} defaultOpen>
        <SettingsTextInput
          {...form.field('userId')}
          label={UsersStrings.FORM.ID}
          placeholder="jsmith"
          helpText={UsersStrings.FORM.USED_USERNAME}
          required
          disabled={!isCreate || isExternal}
        />

        <SettingsTextInput
          {...form.field('firstName')}
          label={UsersStrings.FORM.FIRST_NAME}
          placeholder={UsersStrings.FORM.JOHN_PLACEHOLDER}
          helpText={UsersStrings.FORM.USERS_FIRST_NAME_DISPLAY_PURPOSES}
          required={!isExternal}
          disabled={isExternal}
        />

        <SettingsTextInput
          {...form.field('lastName')}
          label={UsersStrings.FORM.LAST_NAME}
          placeholder={UsersStrings.FORM.SMITH_PLACEHOLDER}
          helpText={UsersStrings.FORM.USERS_LAST_NAME_DISPLAY_PURPOSES}
          required={!isExternal}
          disabled={isExternal}
        />

        <SettingsTextInput
          {...form.field('emailAddress')}
          label={UsersStrings.FORM.EMAIL}
          placeholder="jsmith@example.com"
          type="email"
          helpText={UsersStrings.FORM.USED_NOTIFICATIONS}
          required={!isExternal}
          disabled={isExternal}
        />

        {isCreate && !isExternal && (
          <>
            <SettingsPasswordInput
              {...form.field('password')}
              label={UsersStrings.FORM.PASSWORD}
              helpText={UsersStrings.FORM.SET_INITIAL_PASSWORD_USER_CAN}
              required
              autoComplete="new-password"
            />
            <SettingsPasswordInput
              {...form.field('passwordConfirm')}
              label={UsersStrings.FORM.CONFIRM_PASSWORD}
              helpText={UsersStrings.FORM.RE_ENTER_PASSWORD_CONFIRM}
              required
              autoComplete="new-password"
            />
          </>
        )}

        {!isCreate && !isExternal && (
          <Box mt="4" mb="4">
            {!showPasswordChange ? (
              <SettingsButton
                variant="secondary"
                icon={Key}
                onClick={showPasswordChangeSection}
                testId="change-password-btn"
                data-analytics-id="as-user-change-password"
              >
                {UsersStrings.FORM.CHANGE_PASSWORD}
              </SettingsButton>
            ) : (
              <Box className="user-form__password-change">
                <Flex align="center" justify="between" mb="2">
                  <Text size="2" weight="medium">
                    {UsersStrings.FORM.CHANGE_PASSWORD}
                  </Text>
                  <SettingsButton
                    variant="ghost"
                    size="small"
                    onClick={resetPasswordFields}
                  >
                    {CommonStrings.ACTIONS.CANCEL}
                  </SettingsButton>
                </Flex>
                <SettingsPasswordInput
                  {...form.field('password')}
                  label={UsersStrings.FORM.NEW_PASSWORD}
                  required
                  autoComplete="new-password"
                />
                <SettingsPasswordInput
                  {...form.field('passwordConfirm')}
                  label={UsersStrings.FORM.CONFIRM_NEW_PASSWORD}
                  required
                  autoComplete="new-password"
                />
              </Box>
            )}
          </Box>
        )}
      </SettingsFormSection>

      <SettingsFormSection title={UsersStrings.FORM.ROLES} defaultOpen>
        <SettingsTransferList
          name="roles"
          testId="user-form-roles"
          availableItems={availableRoles}
          selectedItems={selectedRoles}
          onChange={handleRolesChange}
          availableLabel={UsersStrings.FORM.AVAILABLE}
          selectedLabel={UsersStrings.FORM.GRANTED}
          getItemId={(item) => item.id}
          getItemLabel={(item) => item.name}
          helpText={
            (formData.roles ?? []).length > 0
              ? plural(UsersStrings.FORM.ROLES_GRANTED, (formData.roles ?? []).length)
              : undefined
          }
        />
        {form.validationErrors?.roles && (
          <Text size="1" className="user-form__error">
            {form.validationErrors.roles}
          </Text>
        )}

        {isExternal && externalRoles.length > 0 && (
          <Box mt="4">
            <Text
              size="2"
              weight="medium"
              mb="2"
              style={{ display: 'block' }}
            >
              {UsersStrings.FORM.EXTERNALLY_ASSIGNED_ROLES}
            </Text>
            <Box className="user-form__external-roles">
              <Flex wrap="wrap" gap="1">
                {externalRoles.map((roleId) => {
                  const role = allRoles.find((r) => r.id === roleId);
                  const label = role?.name ?? roleId;
                  return (
                    <Box
                      key={roleId}
                      className="user-form__external-role-chip"
                      data-testid={`external-role-${roleId}`}
                    >
                      {label}
                    </Box>
                  );
                })}
              </Flex>
            </Box>
          </Box>
        )}
      </SettingsFormSection>

      <SettingsFormSection
        title={UsersStrings.FORM.PRIVILEGES_SECURITY_TREE}
        defaultOpen
      >
        {rolesDirty && (
          <Box
            className="user-form__preview-hint"
            data-testid="tree-preview-unsaved"
          >
            <Text size="1" color="gray">
              {UsersStrings.FORM.PREVIEW_REFLECTS_UNSAVED_ROLE_CHANGES}
            </Text>
          </Box>
        )}
        {(formData.roles ?? []).length > 0 ? (
          <Box className="user-form__preview-pane">
            <RoleExplorerTree
              tree={previewTree}
              loading={previewLoading}
              onToggleExpand={previewToggleExpand}
              onExpandAll={previewExpandAll}
              onCollapseAll={previewCollapseAll}
              onSearchChange={setPreviewSearchTerm}
            />
          </Box>
        ) : (
          <Box className="user-form__preview-placeholder">
            <Text size="2" color="gray">
              {UsersStrings.FORM.GRANT_LEAST_ONE_ROLE_PREVIEW}
            </Text>
          </Box>
        )}
      </SettingsFormSection>
    </Box>
  );
}

export default UserForm;
