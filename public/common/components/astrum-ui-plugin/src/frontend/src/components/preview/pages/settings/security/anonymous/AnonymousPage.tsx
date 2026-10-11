

import React, { useMemo } from 'react';
import { Box, Flex, Text, } from '@radix-ui/themes';
import * as Dialog from '@radix-ui/react-dialog';
import { Loader2, AlertTriangle, X, } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsCheckbox,
  SettingsTextInput,
  SettingsSelect,
  SettingsAlert,
  SettingsButton,
} from '../../../../shared/form';
import { HelpSection, PageHeader } from '../../../../shared';
import { useAnonymousForm } from './useAnonymousForm';
import { AnonymousPageProps } from './types';

import './AnonymousPage.scss';
import AnonymousStrings from '../../../../constants/pages/settings/security/AnonymousStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

/**
 * AnonymousPage - Anonymous Access settings configuration page for Preview UI
 *
 * Allows enabling/disabling anonymous access and configuring the anonymous user.
 * Uses XState form machine for state management.
 */
export function AnonymousPage({ className }: AnonymousPageProps) {
  // XState form hook handles load, save, dirty tracking, toast, validation, disable confirmation
  const anonForm = useAnonymousForm();

  const canUpdate = ExtJS.checkPermission('astrum:settings:update');

  // Convert realm types to options format for SettingsSelect
  const realmOptions = useMemo(() => {
    return anonForm.realmTypes.map((realm) => ({
      value: realm.id,
      label: realm.name,
    }));
  }, [anonForm.realmTypes]);

  // Loading state
  if (anonForm.isLoading) {
    return (
      <Box className={`anonymous-page ${className || ''}`.trim()} data-testid="anonymous-page">
        <Flex align="center" justify="center" className="anonymous-page__loading">
          <Loader2 size={24} className="anonymous-page__spinner" />
          <Text size="2">{AnonymousStrings.PAGE.LOADING_ANONYMOUS_ACCESS_SETTINGS}</Text>
        </Flex>
      </Box>
    );
  }

  // Read-only view for users without update permission
  if (!canUpdate) {
    return (
      <Box
        className={`anonymous-page ${className || ''}`.trim()}
        data-testid="anonymous-page"
        data-mode="view"
        px={{ initial: '4', md: '6', lg: '6' }}
        py={{ initial: '4', md: '5', lg: '6' }}
      >
        <Box mb="4">
          <PageHeader
            title={AnonymousStrings.PAGE.ANONYMOUS_ACCESS}
            description={AnonymousStrings.PAGE.CONFIGURE_ANONYMOUS_USER_ACCESS_SETTINGS}
          
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: AnonymousStrings.PAGE.ANONYMOUS }
          ]}
/>
        </Box>

        <SettingsFormSection title={AnonymousStrings.PAGE.CURRENT_SETTINGS} defaultOpen>
          <Box className="anonymous-page__readonly">
            <Flex className="anonymous-page__row">
              <Text size="2" weight="medium" className="anonymous-page__label">{AnonymousStrings.PAGE.ANONYMOUS_ACCESS}</Text>
              <Text size="2">{anonForm.formData.enabled ? AnonymousStrings.PAGE.ENABLED : AnonymousStrings.PAGE.DISABLED}</Text>
            </Flex>
            <Flex className="anonymous-page__row">
              <Text size="2" weight="medium" className="anonymous-page__label">{AnonymousStrings.PAGE.USERNAME}</Text>
              <Text size="2">{anonForm.formData.userId}</Text>
            </Flex>
            <Flex className="anonymous-page__row">
              <Text size="2" weight="medium" className="anonymous-page__label">{AnonymousStrings.PAGE.REALM}</Text>
              <Text size="2">
                {anonForm.realmTypes.find((r) => r.id === anonForm.formData.realmName)?.name || anonForm.formData.realmName}
              </Text>
            </Flex>
          </Box>
        </SettingsFormSection>
      </Box>
    );
  }

  return (
    <Box
      className={`anonymous-page ${className || ''}`.trim()}
      data-testid="anonymous-page"
      data-mode="edit"
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      <Box mb="4">
        <PageHeader
          title={AnonymousStrings.PAGE.ANONYMOUS_ACCESS}
          description={AnonymousStrings.PAGE.CONFIGURE_ANONYMOUS_USER_ACCESS_SETTINGS}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: AnonymousStrings.PAGE.ANONYMOUS_ACCESS }
          ]}
        />
      </Box>

      {/* Alerts */}
      {anonForm.loadError && (
        <Box className="anonymous-page__alerts">
          <SettingsAlert type="error">
            {anonForm.loadError}
          </SettingsAlert>
        </Box>
      )}

      {/* Form */}
      <SettingsForm
        onSubmit={anonForm.handleSubmit}
        onCancel={anonForm.handleDiscard}
        loading={anonForm.isSaving}
        pristine={anonForm.isPristine}
        testId="anonymous-form"
        submitAnalyticsId="as-anonymous-save"
        data-loading={anonForm.isSaving ? 'true' : 'false'}
        data-dirty={!anonForm.isPristine ? 'true' : 'false'}
        data-submitting={anonForm.isSaving ? 'true' : 'false'}
        data-valid={!anonForm.hasValidationErrors ? 'true' : 'false'}
      >
        <SettingsFormSection title={AnonymousStrings.PAGE.ANONYMOUS_ACCESS_CONFIGURATION} defaultOpen>
          <Text size="2" className="anonymous-page__section-description">
            {AnonymousStrings.PAGE.WHEN_ENABLED_ANONYMOUS_USERS_CAN}
          </Text>

          <SettingsCheckbox
            {...anonForm.checkbox('enabled')}
            label={AnonymousStrings.PAGE.ALLOW_ANONYMOUS_USERS_ACCESS_SERVER}
            description={AnonymousStrings.PAGE.ENABLE_ANONYMOUS_ACCESS_REPOSITORY_CONTENT}
            analyticsId="as-anonymous-toggle"
          />

          <SettingsTextInput
            {...anonForm.field('userId')}
            label={AnonymousStrings.PAGE.USERNAME}
            helpText={AnonymousStrings.PAGE.USERNAME_USED_ANONYMOUS_ACCESS}
            required
            placeholder="anonymous"
          />

          <SettingsSelect
            name="realmName"
            label={AnonymousStrings.PAGE.REALM}
            value={anonForm.formData.realmName}
            onChange={(value: string) => anonForm.handleChange('realmName', value)}
            options={realmOptions}
            helpText={AnonymousStrings.PAGE.SECURITY_REALM_USE_ANONYMOUS_AUTHENTICATION}
            required
          />
        </SettingsFormSection>

        {/* Help Section */}
        <HelpSection
          title={AnonymousStrings.PAGE.ABOUT_ANONYMOUS_ACCESS}
          content={AnonymousStrings.PAGE.ANONYMOUS_ACCESS_ALLOWS_USERS_BROWSE}
          docLink={{
            label: CommonStrings.ACTIONS.VIEW_DOCUMENTATION,
            href: 'https://help.globalart.dev/en/anonymous-access.html',
          }}
        />
      </SettingsForm>

      {/* Confirmation Modal for Disabling Anonymous Access */}
      <Dialog.Root open={anonForm.showDisableConfirm} onOpenChange={() => anonForm.handleCancelDisable()}>
        <Dialog.Portal>
          <Dialog.Overlay className="anonymous-page__modal-overlay" />
          <Dialog.Content className="anonymous-page__modal" data-testid="anonymous-disable-modal">
            <Dialog.Title className="anonymous-page__modal-title">
              <Flex align="center" gap="2">
                <AlertTriangle size={20} className="anonymous-page__modal-icon" />
                {AnonymousStrings.PAGE.DISABLE_ANONYMOUS_ACCESS}
              </Flex>
            </Dialog.Title>
            <Dialog.Description className="anonymous-page__modal-description">
              {AnonymousStrings.PAGE.DISABLING_ANONYMOUS_ACCESS_REQUIRE_AUTHENTICATION}
            </Dialog.Description>
            <Flex gap="3" justify="end" className="anonymous-page__modal-actions">
              <SettingsButton
                variant="secondary"
                onClick={anonForm.handleCancelDisable}
                testId="anonymous-disable-cancel"
              >
                {CommonStrings.ACTIONS.CANCEL}
              </SettingsButton>
              <SettingsButton
                variant="danger"
                onClick={anonForm.handleConfirmDisable}
                testId="anonymous-disable-confirm"
              >
                {AnonymousStrings.PAGE.DISABLE_ANONYMOUS_ACCESS}
              </SettingsButton>
            </Flex>
            <Dialog.Close asChild>
              <button
                type="button"
                className="anonymous-page__modal-close"
                aria-label={CommonStrings.ACTIONS.CLOSE}
                onClick={anonForm.handleCancelDisable}
              >
                <X size={16} />
              </button>
            </Dialog.Close>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </Box>
  );
}

export default AnonymousPage;
