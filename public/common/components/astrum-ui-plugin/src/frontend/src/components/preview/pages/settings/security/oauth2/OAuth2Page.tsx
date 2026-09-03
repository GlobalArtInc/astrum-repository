

import React, { useState } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2, ShieldCheck } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import ValidationUtils from '../../../../../../interface/ValidationUtils';
import UIStrings from '../../../../../../constants/UIStrings';

import { PageHeader } from '../../../../shared';
import {
  SettingsForm,
  SettingsTextInput,
  SettingsPasswordInput,
  SettingsTextArea,
  SettingsFormSection,
  SettingsCheckbox,
  SettingsButton,
} from '../../../../shared/form';
import { CertificateViewDialog } from '../../repository/repositories/facets/CertificateViewDialog';
import { useOAuth2Form } from './useOAuth2Form';
import { OAuth2PageProps } from './types';

import './OAuth2Page.scss';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

export function OAuth2Page({ className }: OAuth2PageProps) {
  const { data, isLoading, isSaving, isPristine, loadError, saveError, field, checkbox, submit, reset } =
    useOAuth2Form();

  const canUpdate = ExtJS.checkPermission('astrum:settings:update');
  const canReadTruststore = ExtJS.checkPermission('astrum:ssl-truststore:read');
  const [showCertDialog, setShowCertDialog] = useState(false);

  // Matches Classic, which passes the token URL as UseAstrumTruststore's
  // remoteUrl and gates on ValidationUtils.isSecureUrl — the certificate being
  // trusted is the one presented by the token endpoint.
  const idpTokenUrl = data.idpTokenUrl;
  const hasSecureTokenUrl = ValidationUtils.isSecureUrl(idpTokenUrl);

  if (isLoading) {
    return (
      <Box className={`oauth2-page ${className || ''}`.trim()}>
        <Flex align="center" justify="center" className="oauth2-page__loading">
          <Loader2 size={24} className="oauth2-page__spinner" />
          <Text size="2">Loading OAuth2 configuration...</Text>
        </Flex>
      </Box>
    );
  }

  return (
    <Box
      className={`oauth2-page ${className || ''}`.trim()}
      data-testid="oauth2-page"
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      <Box mb="4">
        <PageHeader
          title="OAuth2"
          description="Configure OpenID Connect (OIDC) authentication settings"
          breadcrumbs={[
            { label: 'Settings', onClick: () => navigateTo('#preview/admin/settings') },
            { label: 'OAuth2' }
          ]}
        />
      </Box>
      <SettingsForm
        onSave={submit}
        onCancel={reset}
        dirty={!isPristine}
        saving={isSaving}
        error={saveError || loadError || undefined}
        showActions={canUpdate}
        externalDirtyTracking
        testId="oauth2-settings-form"
      >
        <SettingsFormSection title="OIDC Settings">
          {data.redirectUri && (
            <Box>
              <Text as="div" size="2" weight="medium">Redirect URI</Text>
              <Text as="div" size="2" className="oauth2-page__redirect-uri" data-testid="oauth2-redirect-uri">
                {data.redirectUri}
              </Text>
              <Text as="div" size="1" color="gray">
                Register this redirect URI for the client in your identity provider.
              </Text>
            </Box>
          )}
          <SettingsTextInput
            {...field('issuerUrl')}
            label="Issuer URL"
            helpText="For example https://sso.example.com. Endpoints and keys are read from its .well-known/openid-configuration; the endpoint fields below override them."
            disabled={!canUpdate}
          />
          <SettingsPasswordInput
            {...field('clientId')}
            label="Client ID"
            helpText="The client ID registered with your identity provider"
            required
            disabled={!canUpdate}
          />
          <SettingsPasswordInput
            {...field('clientSecret')}
            label="Client Secret"
            helpText="The client secret for authentication. Leave empty for a public client that uses PKCE only."
            disabled={!canUpdate}
            autoComplete="new-password"
          />
          <SettingsTextInput
            {...field('idpAuthorizationUrl')}
            label="IDP Authorization URL"
            helpText="The authorization endpoint of your identity provider"
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('idpLogoutUrl')}
            label="IDP Logout URL"
            helpText="The logout endpoint of your identity provider"
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('idpTokenUrl')}
            label="IDP Token URL"
            helpText="The token endpoint of your identity provider"
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('idpJwksUrl')}
            label="IDP JWKS URL"
            helpText="The JWKS endpoint for token verification"
            disabled={!canUpdate}
          />

          {/*
            Truststore control. Kept always-visible-but-disabled (rather than
            hidden) to match Classic, so the setting never silently disappears
            from a configuration that already has it enabled.
          */}
          <Box>
            <SettingsCheckbox
              {...checkbox('useTrustStore')}
              label={UIStrings.USE_TRUST_STORE.LABEL}
              description={
                hasSecureTokenUrl
                  ? UIStrings.USE_TRUST_STORE.DESCRIPTION
                  : UIStrings.USE_TRUST_STORE.NOT_SECURE_URL
              }
              disabled={!canUpdate || !hasSecureTokenUrl}
              analyticsId="as-oauth2-toggle-truststore"
            />
            <Box mt="2" ml="6">
              <SettingsButton
                variant="secondary"
                size="small"
                icon={ShieldCheck}
                onClick={() => setShowCertDialog(true)}
                disabled={!canReadTruststore || !hasSecureTokenUrl}
              >
                {UIStrings.USE_TRUST_STORE.VIEW_CERTIFICATE}
              </SettingsButton>
            </Box>
          </Box>
        </SettingsFormSection>

        <SettingsFormSection title="Claim Mappings">
          <SettingsTextInput
            {...field('usernameClaim')}
            label="Username Claim"
            helpText="The claim that contains the username"
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('firstNameClaim')}
            label="First Name Claim"
            helpText="The claim that contains the first name"
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('lastNameClaim')}
            label="Last Name Claim"
            helpText="The claim that contains the last name"
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('emailClaim')}
            label="Email Claim"
            helpText="The claim that contains the email address"
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('groupsClaim')}
            label="Groups Claim"
            helpText="The claim that contains the group memberships"
            disabled={!canUpdate}
          />
        </SettingsFormSection>

        <SettingsFormSection title="JWT Settings">
          <SettingsTextInput
            {...field('idpJwsAlgorithm')}
            label="JWS Algorithm"
            helpText="The signing algorithm used by the IDP (e.g., RS256)"
            disabled={!canUpdate}
          />
          <SettingsTextArea
            {...field('idpJwks')}
            label="JWKS (Optional)"
            helpText="Optional JSON Web Key Set for offline verification"
            disabled={!canUpdate}
            rows={4}
          />
        </SettingsFormSection>

        <SettingsFormSection title="Advanced Settings">
          <SettingsTextArea
            {...field('authorizationCustomParams')}
            label="Authorization Custom Parameters"
            helpText="Custom parameters for authorization requests (JSON format)"
            disabled={!canUpdate}
            rows={3}
          />
          <SettingsTextArea
            {...field('tokenRequestCustomParams')}
            label="Token Request Custom Parameters"
            helpText="Custom parameters for token requests (JSON format)"
            disabled={!canUpdate}
            rows={3}
          />
          <SettingsTextArea
            {...field('exactMatchClaims')}
            label="Exact Match Claims"
            helpText="Claims that must match exactly (JSON format)"
            disabled={!canUpdate}
            rows={3}
          />
        </SettingsFormSection>
      </SettingsForm>

      {showCertDialog && (
        <CertificateViewDialog remoteUrl={idpTokenUrl} onClose={() => setShowCertDialog(false)} />
      )}
    </Box>
  );
}

export default OAuth2Page;
