

import React, { useState } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2, Info, ExternalLink, ShieldCheck } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { PageHeader } from '../../../../shared';
import {
  SettingsForm,
  SettingsFormSection,
  SettingsCheckbox,
  SettingsTextInput,
  SettingsPasswordInput,
  SettingsButton,
} from '../../../../shared/form';
import SslCertificateDetailsModal from '../../../../../widgets/SslCertificateDetailsModal/SslCertificateDetailsModal';
import { EmailVerify } from './EmailVerify';
import { useEmailForm } from './useEmailForm';
import { useEmailApi } from './useEmailApi';
import { EmailConfiguration, EmailPageProps } from './types';

import './EmailPage.scss';
import EmailStrings from '../../../../constants/pages/settings/system/EmailStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

/**
 * EmailPage - Email Server configuration page for Preview UI
 *
 * Configures SMTP settings for outgoing email notifications.
 * Uses XState form machine for state management.
 */
export function EmailPage({ className }: EmailPageProps) {
  // XState form hook handles load, save, dirty tracking, toast, validation
  const form = useEmailForm();

  // Email API for verification only
  const { verifying, sendVerificationEmail } = useEmailApi();

  const isCloud = ExtJS.state?.().getValue?.('isCloud', false) ?? false;
  const canUpdate = ExtJS.checkPermission('astrum:settings:update');
  const formData = form.data as EmailConfiguration;

  const [showCertModal, setShowCertModal] = useState(false);

  const canReadTruststore = ExtJS.checkPermission('astrum:ssl-truststore:read');
  const remoteUrl = formData.host && formData.port ? `https://${formData.host}:${formData.port}` : '';

  if (isCloud) {
    return null;
  }

  // Loading state
  if (form.isLoading) {
    return (
      <Box className={`email-page ${className || ''}`.trim()}>
        <Flex align="center" justify="center" className="email-page__loading">
          <Loader2 size={24} className="email-page__spinner" />
          <Text size="2">{EmailStrings.PAGE.LOADING_EMAIL_SERVER_SETTINGS}</Text>
        </Flex>
      </Box>
    );
  }

  // Read-only view for users without update permission
  if (!canUpdate) {
    return (
      <Box className={`email-page ${className || ''}`.trim()}>
        <PageHeader
          title={EmailStrings.PAGE.EMAIL_SERVER}
          description={EmailStrings.PAGE.CONFIGURE_OUTGOING_EMAIL_SERVER_SETTINGS}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: EmailStrings.PAGE.EMAIL_SERVER },
          ]}
        />

        <SettingsFormSection title={EmailStrings.PAGE.CURRENT_SETTINGS}>
          <Box className="email-page__readonly">
            <Flex className="email-page__row">
              <Text size="2" weight="medium" className="email-page__label">{EmailStrings.PAGE.ENABLED}</Text>
              <Text size="2">{formData.enabled ? EmailStrings.PAGE.YES : EmailStrings.PAGE.NO}</Text>
            </Flex>
            <Flex className="email-page__row">
              <Text size="2" weight="medium" className="email-page__label">{EmailStrings.PAGE.SMTP_HOST}</Text>
              <Text size="2">{formData.host || EmailStrings.PAGE.NOT_CONFIGURED}</Text>
            </Flex>
            <Flex className="email-page__row">
              <Text size="2" weight="medium" className="email-page__label">{EmailStrings.PAGE.SMTP_PORT}</Text>
              <Text size="2">{formData.port}</Text>
            </Flex>
            <Flex className="email-page__row">
              <Text size="2" weight="medium" className="email-page__label">{EmailStrings.PAGE.FROM_ADDRESS}</Text>
              <Text size="2">{formData.fromAddress || EmailStrings.PAGE.NOT_CONFIGURED}</Text>
            </Flex>
          </Box>
        </SettingsFormSection>
      </Box>
    );
  }

  return (
    <Box
      className={`email-page ${className || ''}`.trim()}
      data-testid="email-page"
      data-loading={form.isSaving ? 'true' : 'false'}
    >
      <PageHeader
        title={EmailStrings.PAGE.EMAIL_SERVER}
        description={EmailStrings.PAGE.CONFIGURE_OUTGOING_EMAIL_SERVER_SETTINGS}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
          { label: EmailStrings.PAGE.EMAIL_SERVER },
        ]}
      />

      {/* Scrollable content area */}
      <Box className="email-page__content">
        <SettingsForm
          testId="email-form"
          onSubmit={() => form.submit()}
          onCancel={() => form.reset()}
          loading={form.isSaving}
          pristine={form.isPristine}
          cancelDisabled={form.isPristine}
          error={form.saveError || undefined}
          submitAnalyticsId="as-email-save"
        >
          <SettingsFormSection title={EmailStrings.PAGE.SMTP_CONFIGURATION}>
            <SettingsCheckbox
              {...form.checkbox('enabled')}
              label={EmailStrings.PAGE.ENABLE_EMAIL_SERVER}
              description={EmailStrings.PAGE.ALLOW_SYSTEM_SEND_OUTGOING_EMAIL}
            />

            <SettingsTextInput
              {...form.field('host')}
              label={EmailStrings.PAGE.SMTP_HOST}
              helpText={EmailStrings.PAGE.HOSTNAME_SMTP_SERVER}
              placeholder="smtp.example.com"
              required
            />

            <SettingsTextInput
              name="port"
              label={EmailStrings.PAGE.SMTP_PORT}
              type="number"
              value={formData.port != null ? String(formData.port) : ''}
              // Port needs a numeric value in context but UPDATE event types expect string;
              // a numberField() helper on useForm would remove these casts (ASTRUM-52591 follow-up).
              // 0 when cleared so validateEmailConfig's `port === 0` guard marks it as required.
              onChange={(value: string) => form.send({ type: 'UPDATE', name: 'port', value: value ? parseInt(value, 10) : 0 } as any)}
              onBlur={() => form.send({ type: 'BLUR', name: 'port' } as any)}
              error={form.touched?.port ? form.validationErrors?.port : undefined}
              helpText={EmailStrings.PAGE.PORT_NUMBER_SMTP_SERVER_TYPICALLY}
              min={1}
              max={65535}
              required
            />

            <SettingsCheckbox
              {...form.checkbox('astrumTrustStoreEnabled')}
              label={EmailStrings.PAGE.USE_ASTRUM_TRUST_STORE}
              description={EmailStrings.PAGE.USE_CERTIFICATES_ASTRUM_REPOSITORY_TRUST}
            />

            <Box>
              <SettingsButton
                type="button"
                variant="secondary"
                onClick={() => setShowCertModal(true)}
                disabled={!remoteUrl || !canReadTruststore}
                icon={ShieldCheck}
              >
                {EmailStrings.PAGE.VIEW_CERTIFICATE}
              </SettingsButton>
            </Box>

            {showCertModal && remoteUrl && (
              <SslCertificateDetailsModal
                remoteUrl={remoteUrl}
                onCancel={() => setShowCertModal(false)}
              />
            )}
          </SettingsFormSection>

          <SettingsFormSection title={EmailStrings.PAGE.AUTHENTICATION}>
            <SettingsCheckbox
              {...form.checkbox('useAuthentication')}
              label={EmailStrings.PAGE.ENABLE_AUTHENTICATION}
              description={EmailStrings.PAGE.PROVIDE_CREDENTIALS_SMTP_SERVER_AUTHENTICATION}
            />

            {formData.useAuthentication && (
              <>
                <SettingsTextInput
                  {...form.field('username')}
                  label={EmailStrings.PAGE.USERNAME}
                  helpText={EmailStrings.PAGE.USERNAME_SMTP_AUTHENTICATION}
                  autoComplete="username"
                  placeholder="astrum-smtp-user"
                />
                <SettingsPasswordInput
                  {...form.field('password')}
                  label={EmailStrings.PAGE.PASSWORD}
                  helpText={EmailStrings.PAGE.PASSWORD_SMTP_AUTHENTICATION}
                  autoComplete="new-password"
                  placeholder={form.isPristine && !formData.password ? EmailStrings.PAGE.SAVED_ENTER_CHANGE : undefined}
                />
              </>
            )}
          </SettingsFormSection>

          <SettingsFormSection title={EmailStrings.PAGE.EMAIL_SETTINGS}>
            <SettingsTextInput
              {...form.field('fromAddress')}
              label={EmailStrings.PAGE.FROM_ADDRESS}
              type="email"
              helpText={EmailStrings.PAGE.EMAIL_ADDRESS_USED_SENDER_OUTGOING}
              placeholder="astrum@example.com"
              required
            />

            <SettingsTextInput
              {...form.field('subjectPrefix')}
              label={EmailStrings.PAGE.SUBJECT_PREFIX}
              helpText={EmailStrings.PAGE.OPTIONAL_PREFIX_ADDED_ALL_EMAIL}
              placeholder="[Astrum]"
            />
          </SettingsFormSection>

          <SettingsFormSection
            title={EmailStrings.PAGE.SSL_TLS_OPTIONS}
            collapsible
            defaultCollapsed={!(formData.startTlsEnabled || formData.startTlsRequired || formData.sslOnConnectEnabled || formData.sslCheckServerIdentityEnabled)}
          >
            <SettingsCheckbox
              {...form.checkbox('startTlsEnabled')}
              label={EmailStrings.PAGE.ENABLE_STARTTLS}
              description={EmailStrings.PAGE.ENABLE_STARTTLS_SUPPORT_SECURE_CONNECTIONS}
            />

            <SettingsCheckbox
              {...form.checkbox('startTlsRequired')}
              label={EmailStrings.PAGE.REQUIRE_STARTTLS}
              description={EmailStrings.PAGE.REQUIRE_STARTTLS_CONNECTIONS_CONNECTION_FAIL}
            />

            <SettingsCheckbox
              {...form.checkbox('sslOnConnectEnabled')}
              label={EmailStrings.PAGE.ENABLE_SSL_TLS_CONNECT}
              description={EmailStrings.PAGE.USE_SSL_TLS_ENCRYPTION_IMMEDIATELY}
            />

            <SettingsCheckbox
              {...form.checkbox('sslCheckServerIdentityEnabled')}
              label={EmailStrings.PAGE.SERVER_IDENTITY_CHECK}
              description={EmailStrings.PAGE.VERIFY_SERVER_CERTIFICATE_MATCHES_HOSTNAME}
            />
          </SettingsFormSection>

        </SettingsForm>

        {/* Test Email — outside form to prevent Enter-key from triggering SMTP save */}
        <Box className="email-page__form-aligned" data-testid="email-verify-section">
          <SettingsFormSection title={EmailStrings.PAGE.TEST_EMAIL}>
            <EmailVerify
              onSendTest={sendVerificationEmail}
              loading={verifying}
              disabled={!formData.enabled}
            />
          </SettingsFormSection>
        </Box>

        {/* Help Section */}
        <Box className="email-page__form-aligned">
          <Box className="email-page__help">
            <Flex align="center" gap="2" className="email-page__help-header">
              <Info size={16} />
              <Text size="2" weight="medium">{EmailStrings.PAGE.ABOUT_EMAIL_SERVER}</Text>
            </Flex>
            <Text size="2" className="email-page__help-text">
              {EmailStrings.PAGE.ABOUT_TEXT}{' '}
              {EmailStrings.PAGE.SEE_DOCUMENTATION(
                <a
                  href="https://help.globalart.dev/en/email-server-configuration.html"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="email-page__help-link"
                >
                  {EmailStrings.PAGE.DOCUMENTATION_LINK}
                  <ExternalLink size={12} />
                </a>
              )}
            </Text>
          </Box>
        </Box>

      </Box>
    </Box>
  );
}

export default EmailPage;
