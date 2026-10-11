


import React from 'react';
import { Box, Text, Flex, Button, Callout, Spinner, Badge } from '@radix-ui/themes';
import {
  ExternalLink,
  Copy,
  Trash2,
  AlertCircle,
  CheckCircle2,
  Info,
  Circle,
} from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsTextArea,
  SettingsSelect,
  SettingsButton,
  SettingsAlert,
} from '../../../../shared/form';
import { ConfirmDialog } from '../shared/ConfirmDialog';
import { PageHeader } from '../../../../shared';
import { SamlPageProps } from './types';
import { useSamlForm } from './useSamlForm';
import './SamlPage.scss';
import SamlStrings from '../../../../constants/pages/settings/security/SamlStrings';
import CommonStrings from '../../../../constants/CommonStrings';

// Shared options for tri-state signature validation selects
const SIGNATURE_VALIDATION_OPTIONS = [
  { value: 'default', label: SamlStrings.PAGE.DEFAULT },
  { value: 'true', label: SamlStrings.PAGE.TRUE },
  { value: 'false', label: SamlStrings.PAGE.FALSE },
];

export function SamlPage({ className }: SamlPageProps) {
  const {
    config,
    isConfigured,
    isInitialLoading,
    isBusy,
    isPristine,
    isConfirmingDelete,
    error,
    deleteError,
    fieldError,
    handleFieldChange,
    handleSave,
    handleDiscard,
    requestDelete,
    confirmDelete,
    cancelDelete,
    clearError,
    getAbsoluteMetadataUrl,
    copyMetadataUrl,
  } = useSamlForm();

  // Permission checks
  const canUpdate = ExtJS.checkPermission('astrum:saml:update');

  if (isInitialLoading) {
    return (
      <Box
        className={`saml-page ${className || ''}`}
        p="5"
        data-testid="saml-page"
        data-loading="true"
        aria-busy="true"
        aria-live="polite"
      >
        <Flex align="center" justify="center" py="9" gap="3">
          <Spinner size="3" />
          <Text size="3" color="gray">{SamlStrings.PAGE.LOADING_SAML_CONFIGURATION}</Text>
        </Flex>
      </Box>
    );
  }

  return (
    <Box
      className={`saml-page ${className || ''}`}
      data-testid="saml-page"
      data-loading={isBusy ? 'true' : 'false'}
      data-configured={isConfigured ? 'true' : 'false'}
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      {/* Header */}
      <Box mb="4">
        <PageHeader
          title={SamlStrings.PAGE.SAML}
          description={SamlStrings.PAGE.CONFIGURE_SAML_AUTHENTICATION_IDENTITY_PROVIDER}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#preview/admin/settings'; } },
            { label: 'SAML' }
          ]}
          actions={
            isConfigured ? (
              <Badge color="green" variant="soft" data-testid="saml-badge-configured">
                <CheckCircle2 size={12} />
                {SamlStrings.PAGE.CONFIGURED}
              </Badge>
            ) : (
              <Badge color="gray" variant="soft" data-testid="saml-badge-not-configured">
                <Circle size={12} />
                {SamlStrings.PAGE.NOT_CONFIGURED}
              </Badge>
            )
          }
        />
      </Box>

      {/* Alerts */}
      {error && (
        <SettingsAlert type="error" className="mb-4" onClose={clearError}>
          {error}
        </SettingsAlert>
      )}


      {/* Info callout for unconfigured state */}
      {!isConfigured && canUpdate && (
        <Callout.Root color="blue" variant="soft" mb="4">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>
            {SamlStrings.PAGE.SAML_NOT_YET_CONFIGURED_FILL}
          </Callout.Text>
        </Callout.Root>
      )}

      {/* Read-only notice for users without update permission */}
      {!canUpdate && (
        <Callout.Root color="amber" variant="soft" mb="4">
          <Callout.Icon>
            <AlertCircle size={16} />
          </Callout.Icon>
          <Callout.Text>
            {SamlStrings.PAGE.YOU_DO_NOT_HAVE_PERMISSION}
          </Callout.Text>
        </Callout.Root>
      )}

      <SettingsForm
        title=""
        description={undefined}
        loading={isBusy}
        onSave={canUpdate ? handleSave : undefined}
        onSubmit={undefined}
        onCancel={canUpdate ? handleDiscard : undefined}
        dirty={!isPristine}
        pristine={isPristine}
        externalDirtyTracking
        submitLabel={CommonStrings.ACTIONS.SAVE}
        submitAnalyticsId="as-saml-save"
        cancelAnalyticsId={undefined}
        data-testid="saml-form"
        data-dirty={!isPristine ? 'true' : 'false'}
        data-valid={!fieldError('idpMetadata') && !fieldError('entityId') && !fieldError('usernameAttribute') ? 'true' : 'false'}
        footerExtra={
          isConfigured && canUpdate ? (
            <SettingsButton
              variant="danger"
              onClick={requestDelete}
              disabled={isBusy}
              icon={Trash2}
              testId="saml-delete-button"
            >
              {SamlStrings.PAGE.DELETE_CONFIGURATION}
            </SettingsButton>
          ) : undefined
        }
      >
        {/* Service Provider Metadata Section */}
        {isConfigured && (
          <SettingsFormSection
            title={SamlStrings.PAGE.SERVICE_PROVIDER_METADATA}
            description={SamlStrings.PAGE.SHARE_METADATA_URL_IDENTITY_PROVIDER}
          >
            <Flex align="center" gap="3" className="metadata-url-container">
              <Text size="2" className="metadata-url" data-testid="saml-metadata-url">
                {getAbsoluteMetadataUrl()}
              </Text>
              <Button
                variant="soft"
                size="1"
                onClick={copyMetadataUrl}
                data-testid="saml-metadata-copy"
              >
                <Copy size={14} />
                {SamlStrings.PAGE.COPY}
              </Button>
              <Button
                variant="soft"
                size="1"
                onClick={() => window.open(getAbsoluteMetadataUrl(), '_blank')}
                data-testid="saml-metadata-open"
              >
                <ExternalLink size={14} />
                {SamlStrings.PAGE.OPEN}
              </Button>
            </Flex>
          </SettingsFormSection>
        )}

        {/* Identity Provider Configuration */}
        <SettingsFormSection
          title={SamlStrings.PAGE.IDENTITY_PROVIDER_CONFIGURATION}
          description={SamlStrings.PAGE.CONFIGURE_SAML_IDENTITY_PROVIDER_SETTINGS}
        >
          <SettingsTextArea
            name="idpMetadata"
            label={SamlStrings.PAGE.IDENTITY_PROVIDER_METADATA_XML}
            value={config.idpMetadata || ''}
            onChange={(value) => handleFieldChange('idpMetadata', value)}
            placeholder={SamlStrings.PAGE.PASTE_IDENTITY_PROVIDER_METADATA_XML_PLACEHOLDER}
            rows={10}
            required
            error={fieldError('idpMetadata')}
            helpText={SamlStrings.PAGE.SAML_METADATA_XML_PROVIDED_IDENTITY}
            disabled={!canUpdate}
            className="saml-idp-metadata-textarea"
            data-testid="saml-input-idpMetadata"
          />

          <SettingsTextInput
            name="entityId"
            label={SamlStrings.PAGE.ENTITY_ID}
            value={config.entityId || ''}
            onChange={(value) => handleFieldChange('entityId', value)}
            placeholder="https://your-idp.example.com"
            error={fieldError('entityId')}
            helpText={SamlStrings.PAGE.SAML_SERVICE_PROVIDERS_UNIQUE_IDENTIFYING}
            disabled={!canUpdate}
            data-testid="saml-input-entityId"
          />
        </SettingsFormSection>

        {/* Signature Validation */}
        <SettingsFormSection
          title={SamlStrings.PAGE.SIGNATURE_VALIDATION}
          description={SamlStrings.PAGE.CONFIGURE_SIGNATURE_VALIDATION_SETTINGS}
        >
          <SettingsSelect
            name="validateResponseSignature"
            label={SamlStrings.PAGE.VALIDATE_RESPONSE_SIGNATURE}
            value={config.validateResponseSignature === null || config.validateResponseSignature === undefined ? 'default' : String(config.validateResponseSignature)}
            onChange={(value) => handleFieldChange('validateResponseSignature', value === 'default' ? null : value === 'true')}
            options={SIGNATURE_VALIDATION_OPTIONS}
            helpText={SamlStrings.PAGE.REQUIRE_VALID_SIGNATURE_SAML_RESPONSES}
            disabled={!canUpdate}
            data-testid="saml-select-validateResponseSignature"
          />

          <SettingsSelect
            name="validateAssertionSignature"
            label={SamlStrings.PAGE.VALIDATE_ASSERTION_SIGNATURE}
            value={config.validateAssertionSignature === null || config.validateAssertionSignature === undefined ? 'default' : String(config.validateAssertionSignature)}
            onChange={(value) => handleFieldChange('validateAssertionSignature', value === 'default' ? null : value === 'true')}
            options={SIGNATURE_VALIDATION_OPTIONS}
            helpText={SamlStrings.PAGE.REQUIRE_VALID_SIGNATURE_SAML_ASSERTIONS}
            disabled={!canUpdate}
            data-testid="saml-select-validateAssertionSignature"
          />
        </SettingsFormSection>

        {/* Attribute Mapping */}
        <SettingsFormSection
          title={SamlStrings.PAGE.ATTRIBUTE_MAPPING}
          description={SamlStrings.PAGE.MAP_SAML_ATTRIBUTES_USER_PROPERTIES}
        >
          <SettingsTextInput
            name="usernameAttribute"
            label={SamlStrings.PAGE.USERNAME_ATTRIBUTE}
            value={config.usernameAttribute || ''}
            onChange={(value) => handleFieldChange('usernameAttribute', value)}
            placeholder="e.g., NameID, uid, email"
            required
            error={fieldError('usernameAttribute')}
            helpText={SamlStrings.PAGE.SAML_ATTRIBUTE_NAME_CONTAINING_USERNAME}
            disabled={!canUpdate}
            data-testid="saml-input-usernameAttribute"
          />

          <SettingsTextInput
            name="firstNameAttribute"
            label={SamlStrings.PAGE.FIRST_NAME_ATTRIBUTE}
            value={config.firstNameAttribute || ''}
            onChange={(value) => handleFieldChange('firstNameAttribute', value)}
            placeholder="e.g., firstName, givenName"
            helpText={SamlStrings.PAGE.FIRST_NAME_ATTRIBUTE_HELP}
            disabled={!canUpdate}
            data-testid="saml-input-firstNameAttribute"
          />

          <SettingsTextInput
            name="lastNameAttribute"
            label={SamlStrings.PAGE.LAST_NAME_ATTRIBUTE}
            value={config.lastNameAttribute || ''}
            onChange={(value) => handleFieldChange('lastNameAttribute', value)}
            placeholder="e.g., lastName, sn, surname"
            helpText={SamlStrings.PAGE.LAST_NAME_ATTRIBUTE_HELP}
            disabled={!canUpdate}
            data-testid="saml-input-lastNameAttribute"
          />

          <SettingsTextInput
            name="emailAttribute"
            label={SamlStrings.PAGE.EMAIL_ATTRIBUTE}
            value={config.emailAttribute || ''}
            onChange={(value) => handleFieldChange('emailAttribute', value)}
            placeholder="e.g., email, mail"
            helpText={SamlStrings.PAGE.EMAIL_ATTRIBUTE_HELP}
            disabled={!canUpdate}
            data-testid="saml-input-emailAttribute"
          />

          <SettingsTextInput
            name="groupsAttribute"
            label={SamlStrings.PAGE.ROLES_GROUPS}
            value={config.groupsAttribute || ''}
            onChange={(value) => handleFieldChange('groupsAttribute', value)}
            placeholder="e.g., groups, memberOf"
            helpText={SamlStrings.PAGE.MAP_IDENTITY_PROVIDER_ROLES_GROUPS}
            disabled={!canUpdate}
            data-testid="saml-input-groupsAttribute"
          />
        </SettingsFormSection>


      </SettingsForm>

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={isConfirmingDelete}
        testId="delete-saml-config-dialog"
        onOpenChange={(open) => { if (!open) cancelDelete(); }}
        title={SamlStrings.PAGE.DELETE_SAML_CONFIGURATION}
        message={SamlStrings.PAGE.DELETE_CONFIRM}
        confirmLabel={SamlStrings.PAGE.DELETE_CONFIGURATION}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="danger"
        onConfirm={confirmDelete}
      >
        {/* A failed delete keeps the dialog open with the error shown here. */}
        {deleteError && (
          <Box mt="3">
            <SettingsAlert type="error">{deleteError}</SettingsAlert>
          </Box>
        )}
      </ConfirmDialog>
    </Box>
  );
}

export default SamlPage;
