
import React from 'react';
import { Box, Flex, Text, Heading, AlertDialog, Button, TextField } from '@radix-ui/themes';
import * as Dialog from '@radix-ui/react-dialog';
import { Loader2, RefreshCw, AlertTriangle } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { Permissions } from '../../../../../../constants/Permissions';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsCheckbox,
  SettingsTextInput,
  SettingsButton,
  SettingsAlert,
} from '../../../../shared/form';
import { PageHeader } from '../../../../shared';
import { useUserTokensForm } from './useUserTokensForm';
import { RESET_CONFIRMATION_STRING } from './types';

import './UserTokensPage.scss';
import UserTokensStrings from '../../../../constants/pages/settings/security/UserTokensStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import { formatDuration } from '../../../../shared/utils/formatLocale';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

/**
 * UserTokensPage - User Token settings configuration page for Preview UI
 */
export function UserTokensPage() {
  const {
    data,
    pristineData,
    expirationDaysError,
    isPristine,
    isLoading,
    isSaving,
    isResetting,
    error,
    showExpirationWarning,
    showResetModal,
    resetConfirmationInput,
    resetConfirmationError,
    handleChange,
    handleSubmit,
    handleDiscard,
    confirmSave,
    cancelSave,
    requestReset,
    setResetConfirmation,
    confirmReset,
    cancelReset,
    clearError,
  } = useUserTokensForm();

  const isBusy = isSaving || isResetting;

  const canUpdate = ExtJS.checkPermission(Permissions.USER_TOKENS_SETTINGS.UPDATE);
  const canDelete = ExtJS.checkPermission(Permissions.USER_TOKENS_USERS.DELETE);

  // Loading state
  if (isLoading) {
    return (
      <Box className="user-tokens-page" data-testid="user-tokens-page" data-loading="true">
        <Flex align="center" justify="center" className="user-tokens-page__loading">
          <Loader2 size={24} className="user-tokens-page__spinner" />
          <Text size="2">{UserTokensStrings.PAGE.LOADING_USER_TOKEN_SETTINGS}</Text>
        </Flex>
      </Box>
    );
  }

  // Read-only view for users without update permission
  if (!canUpdate) {
    return (
      <Box
        className="user-tokens-page"
        data-testid="user-tokens-page"
        data-view="readonly"
        px={{ initial: '4', md: '6', lg: '6' }}
        py={{ initial: '4', md: '5', lg: '6' }}
      >
        <Box mb="4">
          <PageHeader
            title={UserTokensStrings.PAGE.USER_TOKENS}
            description={UserTokensStrings.PAGE.CONFIGURE_USER_TOKEN_SETTINGS}
          
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: UserTokensStrings.PAGE.USER_TOKENS }
          ]}
/>
        </Box>

        <SettingsFormSection title={UserTokensStrings.PAGE.CURRENT_SETTINGS} defaultOpen>
          <Box className="user-tokens-page__readonly">
            <Flex className="user-tokens-page__row">
              <Text size="2" weight="medium" className="user-tokens-page__label">{UserTokensStrings.PAGE.USER_TOKENS}</Text>
              <Text size="2">{data.enabled ? UserTokensStrings.PAGE.ENABLED : UserTokensStrings.PAGE.DISABLED}</Text>
            </Flex>
            <Flex className="user-tokens-page__row">
              <Text size="2" weight="medium" className="user-tokens-page__label">{UserTokensStrings.PAGE.REPOSITORY_AUTHENTICATION}</Text>
              <Text size="2">{data.protectContent ? UserTokensStrings.PAGE.REQUIRED : UserTokensStrings.PAGE.NOT_REQUIRED}</Text>
            </Flex>
            <Flex className="user-tokens-page__row">
              <Text size="2" weight="medium" className="user-tokens-page__label">{UserTokensStrings.PAGE.TOKEN_EXPIRATION}</Text>
              <Text size="2">
                {data.expirationEnabled
                  ? formatDuration(data.expirationDays, 'day')
                  : UserTokensStrings.PAGE.NEVER_EXPIRES}
              </Text>
            </Flex>
          </Box>
        </SettingsFormSection>
      </Box>
    );
  }

  return (
    <Box
      className="user-tokens-page"
      data-testid="user-tokens-page"
      data-view="edit"
      data-loading={isBusy ? 'true' : 'false'}
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      {/* Header */}
      <Box mb="4">
        <PageHeader
          title={UserTokensStrings.PAGE.USER_TOKENS}
          description={UserTokensStrings.PAGE.CONFIGURE_USER_TOKEN_SETTINGS_REPOSITORY}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: UserTokensStrings.PAGE.USER_TOKENS }
          ]}
        />
      </Box>

      {/* Alerts */}
      {error && (
        <Box className="user-tokens-page__alerts">
          <SettingsAlert type="error" onClose={clearError}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Form */}
      <SettingsForm
        title=""
        onSubmit={handleSubmit}
        onCancel={handleDiscard}
        loading={isSaving}
        pristine={isPristine}
        submitDisabled={!!expirationDaysError}
        showActions={true}
        testId="user-tokens-form"
        data-submitting={isSaving ? 'true' : 'false'}
        data-valid={!expirationDaysError ? 'true' : 'false'}
      >
        {/* User Tokens Enable/Disable */}
        <SettingsFormSection title={UserTokensStrings.PAGE.USER_TOKEN_CONFIGURATION} defaultOpen>
          <Text size="2" className="user-tokens-page__help-text">
            {UserTokensStrings.PAGE.USER_TOKENS_ALLOW_USERS_AUTHENTICATE}
          </Text>

          <SettingsCheckbox
            name="enabled"
            label={UserTokensStrings.PAGE.ENABLE_USER_TOKENS}
            checked={data.enabled}
            onChange={(checked) => handleChange('enabled', checked)}
            description={UserTokensStrings.PAGE.ALLOW_USERS_GENERATE_USE_AUTHENTICATION}
          />
        </SettingsFormSection>

        {/* Repository Authentication */}
        <SettingsFormSection title={UserTokensStrings.PAGE.REPOSITORY_AUTHENTICATION} defaultOpen={data.enabled}>
          <SettingsCheckbox
            name="protectContent"
            label={UserTokensStrings.PAGE.REQUIRE_USER_TOKENS_REPOSITORY_AUTHENTICATION}
            checked={data.protectContent}
            onChange={(checked) => handleChange('protectContent', checked)}
            description={UserTokensStrings.PAGE.WHEN_ENABLED_USERS_MUST_USE}
            disabled={!data.enabled}
          />
        </SettingsFormSection>

        {/* Token Expiration */}
        <SettingsFormSection title={UserTokensStrings.PAGE.TOKEN_EXPIRATION} defaultOpen={data.enabled}>
          <SettingsCheckbox
            name="expirationEnabled"
            label={UserTokensStrings.PAGE.ENABLE_TOKEN_EXPIRATION}
            checked={data.expirationEnabled}
            onChange={(checked) => handleChange('expirationEnabled', checked)}
            description={UserTokensStrings.PAGE.AUTOMATICALLY_EXPIRE_TOKENS_AFTER_SPECIFIED}
            disabled={!data.enabled}
          />

          {data.expirationEnabled && (
            <SettingsTextInput
              name="expirationDays"
              label={UserTokensStrings.PAGE.TOKEN_EXPIRATION_DAYS}
              type="number"
              value={data.expirationDays}
              onChange={(value) => handleChange('expirationDays', parseInt(value, 10) || 0)}
              error={expirationDaysError}
              helpText={UserTokensStrings.PAGE.NUMBER_DAYS_BEFORE_TOKENS_EXPIRE}
              min={1}
              max={999}
              disabled={!data.enabled}
              required
            />
          )}
        </SettingsFormSection>

        {/* Reset All Tokens */}
        {canDelete && data.enabled && pristineData.enabled && (
          <SettingsFormSection title={UserTokensStrings.PAGE.RESET_ALL_TOKENS}>
            <Text size="2" className="user-tokens-page__warning-text">
              <AlertTriangle size={16} className="user-tokens-page__warning-icon" />
              {UserTokensStrings.PAGE.RESETTING_ALL_TOKENS_INVALIDATE_EVERY}
            </Text>
            <SettingsButton
              variant="danger"
              onClick={requestReset}
              disabled={isBusy}
              icon={RefreshCw}
            >
              {UserTokensStrings.PAGE.RESET_ALL_USER_TOKENS}
            </SettingsButton>
          </SettingsFormSection>
        )}
      </SettingsForm>

      {/* Reset Confirmation Modal — aligned with the Delete Repository destructive
          pattern (Radix Themes AlertDialog). Confirmation stays machine-driven:
          confirm-on-click validates the typed phrase and, if wrong, keeps the modal
          open with an inline error rather than disabling the button. */}
      <AlertDialog.Root open={showResetModal} onOpenChange={(open) => !open && cancelReset()}>
        <AlertDialog.Content maxWidth="450px" data-testid="user-tokens-reset-modal">
          <AlertDialog.Title size="5">{UserTokensStrings.PAGE.RESET_ALL_CONFIRM_TITLE}</AlertDialog.Title>

          <AlertDialog.Description size="2" mb="4">
            {UserTokensStrings.PAGE.ACTION_INVALIDATE_ALL_EXISTING_USER}
          </AlertDialog.Description>

          {/* Highlighted warning box showing the exact phrase the user must type.
              Sourced from RESET_CONFIRMATION_STRING so the displayed phrase, the
              placeholder, and the machine guard stay in sync. */}
          <Box
            p="3"
            mb="4"
            style={{
              backgroundColor: 'var(--red-2)',
              border: '1px solid var(--red-6)',
              borderRadius: '6px',
            }}
          >
            <Text size="2" weight="medium" style={{ display: 'block' }}>
              {RESET_CONFIRMATION_STRING}
            </Text>
            <Text size="1" color="gray">
              {UserTokensStrings.PAGE.TYPE_CONFIRM_RESET}
            </Text>
          </Box>

          {/* Acknowledgement input */}
          <Box mb="4">
            <Flex align="center" gap="1" mb="2">
              <Text size="2" weight="bold">
                {UserTokensStrings.PAGE.ACKNOWLEDGEMENT}
              </Text>
              <Text size="2" style={{ color: 'var(--red-11)' }}>
                *
              </Text>
            </Flex>
            <TextField.Root
              size="2"
              value={resetConfirmationInput}
              onChange={(e) => setResetConfirmation(e.target.value)}
              onKeyDown={(e) => {
                // Enter mirrors the confirm click exactly: call confirmReset() and let the
                // machine guard decide. Correct phrase resets; wrong phrase shows the same
                // inline error a click would. No phrase check here.
                if (e.key === 'Enter' && !isResetting) {
                  confirmReset();
                }
              }}
              placeholder={UserTokensStrings.PAGE.TYPE_CONFIRM(RESET_CONFIRMATION_STRING)}
              disabled={isResetting}
              color={resetConfirmationError ? 'red' : undefined}
              aria-label={UserTokensStrings.PAGE.ACKNOWLEDGEMENT_ARIA}
              data-testid="user-tokens-reset-confirmation-input"
              autoFocus
            />
            {resetConfirmationError && (
              <Text size="1" color="red" mt="1" style={{ display: 'block' }} role="alert">
                {resetConfirmationError}
              </Text>
            )}
          </Box>

          {/* Action Buttons — confirm is a plain Button (not AlertDialog.Action) so a
              wrong phrase does not auto-close the dialog; the machine controls closure. */}
          <Flex gap="3" mt="4" justify="end">
            <AlertDialog.Cancel>
              <Button
                variant="surface"
                color="gray"
                size="2"
                data-testid="user-tokens-reset-cancel"
              >
                {CommonStrings.ACTIONS.CANCEL}
              </Button>
            </AlertDialog.Cancel>
            <Button
              variant="solid"
              color="red"
              size="2"
              onClick={confirmReset}
              disabled={isResetting}
              data-testid="user-tokens-reset-confirm"
            >
              {isResetting ? UserTokensStrings.PAGE.RESETTING : UserTokensStrings.PAGE.RESET_ALL_TOKENS}
            </Button>
          </Flex>
        </AlertDialog.Content>
      </AlertDialog.Root>

      {/* Expiration Warning Modal */}
      <Dialog.Root open={showExpirationWarning} onOpenChange={(open) => !open && cancelSave()}>
        <Dialog.Portal>
          <Dialog.Overlay className="user-tokens-page__modal-overlay" />
          <Dialog.Content className="user-tokens-page__modal" data-testid="user-tokens-expiration-modal">
            <Flex align="center" gap="3" className="user-tokens-page__modal-header">
              <AlertTriangle size={24} className="user-tokens-page__modal-icon" />
              <Dialog.Title asChild>
                <Heading as="h2" size="4" weight="medium" className="user-tokens-page__modal-title">
                  {UserTokensStrings.PAGE.TOKEN_EXPIRATION_CHANGE}
                </Heading>
              </Dialog.Title>
            </Flex>
            <Dialog.Description asChild>
              <Box className="user-tokens-page__modal-description">
                <Text size="2" className="user-tokens-page__modal-text">
                  {data.expirationEnabled
                    ? UserTokensStrings.PAGE.ENABLING_TOKEN_EXPIRATION_CAUSE_EXISTING
                    : UserTokensStrings.PAGE.DISABLING_TOKEN_EXPIRATION_ALLOW_ALL}
                </Text>
              </Box>
            </Dialog.Description>
            <Flex gap="4" justify="end" className="user-tokens-page__modal-actions">
              <Dialog.Close asChild>
                <SettingsButton
                  variant="secondary"
                  disabled={isSaving}
                  data-testid="user-tokens-expiration-cancel"
                >
                  {CommonStrings.ACTIONS.CANCEL}
                </SettingsButton>
              </Dialog.Close>
              <SettingsButton
                variant="primary"
                onClick={confirmSave}
                loading={isSaving}
                data-testid="user-tokens-expiration-confirm"
              >
                {UserTokensStrings.PAGE.CONTINUE}
              </SettingsButton>
            </Flex>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </Box>
  );
}

export default UserTokensPage;
