

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
import OAuth2Strings from '../../../../constants/pages/settings/security/OAuth2Strings';
import CommonStrings from '../../../../constants/CommonStrings';

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
          <Text size="2">{OAuth2Strings.PAGE.LOADING_OAUTH2_CONFIGURATION}</Text>
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
          title={OAuth2Strings.PAGE.OPENID_CONNECT}
          description={OAuth2Strings.PAGE.CONFIGURE_OPENID_CONNECT_OIDC_AUTHENTICATION}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: OAuth2Strings.PAGE.OPENID_CONNECT }
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
        <SettingsFormSection title={OAuth2Strings.PAGE.OIDC_SETTINGS}>
          {data.redirectUri && (
            <Box>
              <Text as="div" size="2" weight="medium">{OAuth2Strings.PAGE.REDIRECT_URI}</Text>
              <Text as="div" size="2" className="oauth2-page__redirect-uri" data-testid="oauth2-redirect-uri">
                {data.redirectUri}
              </Text>
              <Text as="div" size="1" color="gray">
                {OAuth2Strings.PAGE.REGISTER_REDIRECT_URI_CLIENT_IDENTITY}
              </Text>
            </Box>
          )}
          <SettingsTextInput
            {...field('issuerUrl')}
            label={OAuth2Strings.PAGE.ISSUER_URL}
            helpText={OAuth2Strings.PAGE.EXAMPLE_HTTPS_SSO_EXAMPLE_COM}
            disabled={!canUpdate}
          />
          <SettingsPasswordInput
            {...field('clientId')}
            label={OAuth2Strings.PAGE.CLIENT_ID}
            helpText={OAuth2Strings.PAGE.CLIENT_ID_REGISTERED_IDENTITY_PROVIDER}
            required
            disabled={!canUpdate}
          />
          <SettingsPasswordInput
            {...field('clientSecret')}
            label={OAuth2Strings.PAGE.CLIENT_SECRET}
            helpText={OAuth2Strings.PAGE.CLIENT_SECRET_AUTHENTICATION_LEAVE_EMPTY}
            disabled={!canUpdate}
            autoComplete="new-password"
          />
          <SettingsTextInput
            {...field('idpAuthorizationUrl')}
            label={OAuth2Strings.PAGE.IDP_AUTHORIZATION_URL}
            helpText={OAuth2Strings.PAGE.AUTHORIZATION_ENDPOINT_IDENTITY_PROVIDER}
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('idpLogoutUrl')}
            label={OAuth2Strings.PAGE.IDP_LOGOUT_URL}
            helpText={OAuth2Strings.PAGE.LOGOUT_ENDPOINT_IDENTITY_PROVIDER}
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('idpTokenUrl')}
            label={OAuth2Strings.PAGE.IDP_TOKEN_URL}
            helpText={OAuth2Strings.PAGE.TOKEN_ENDPOINT_IDENTITY_PROVIDER}
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('idpJwksUrl')}
            label={OAuth2Strings.PAGE.IDP_JWKS_URL}
            helpText={OAuth2Strings.PAGE.JWKS_ENDPOINT_TOKEN_VERIFICATION}
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

        <SettingsFormSection title={OAuth2Strings.PAGE.CLAIM_MAPPINGS}>
          <SettingsTextInput
            {...field('usernameClaim')}
            label={OAuth2Strings.PAGE.USERNAME_CLAIM}
            helpText={OAuth2Strings.PAGE.CLAIM_CONTAINS_USERNAME}
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('firstNameClaim')}
            label={OAuth2Strings.PAGE.FIRST_NAME_CLAIM}
            helpText={OAuth2Strings.PAGE.CLAIM_CONTAINS_FIRST_NAME}
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('lastNameClaim')}
            label={OAuth2Strings.PAGE.LAST_NAME_CLAIM}
            helpText={OAuth2Strings.PAGE.CLAIM_CONTAINS_LAST_NAME}
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('emailClaim')}
            label={OAuth2Strings.PAGE.EMAIL_CLAIM}
            helpText={OAuth2Strings.PAGE.CLAIM_CONTAINS_EMAIL_ADDRESS}
            disabled={!canUpdate}
          />
          <SettingsTextInput
            {...field('groupsClaim')}
            label={OAuth2Strings.PAGE.GROUPS_CLAIM}
            helpText={OAuth2Strings.PAGE.CLAIM_CONTAINS_GROUP_MEMBERSHIPS}
            disabled={!canUpdate}
          />
        </SettingsFormSection>

        <SettingsFormSection title={OAuth2Strings.PAGE.JWT_SETTINGS}>
          <SettingsTextInput
            {...field('idpJwsAlgorithm')}
            label={OAuth2Strings.PAGE.JWS_ALGORITHM}
            helpText={OAuth2Strings.PAGE.SIGNING_ALGORITHM_USED_IDP_E}
            disabled={!canUpdate}
          />
          <SettingsTextArea
            {...field('idpJwks')}
            label={OAuth2Strings.PAGE.JWKS_OPTIONAL}
            helpText={OAuth2Strings.PAGE.OPTIONAL_JSON_WEB_KEY_SET}
            disabled={!canUpdate}
            rows={4}
          />
        </SettingsFormSection>

        <SettingsFormSection title={OAuth2Strings.PAGE.ADVANCED_SETTINGS}>
          <SettingsTextArea
            {...field('authorizationCustomParams')}
            label={OAuth2Strings.PAGE.AUTHORIZATION_CUSTOM_PARAMETERS}
            helpText={OAuth2Strings.PAGE.CUSTOM_PARAMETERS_AUTHORIZATION_REQUESTS_JSON}
            disabled={!canUpdate}
            rows={3}
          />
          <SettingsTextArea
            {...field('tokenRequestCustomParams')}
            label={OAuth2Strings.PAGE.TOKEN_REQUEST_CUSTOM_PARAMETERS}
            helpText={OAuth2Strings.PAGE.CUSTOM_PARAMETERS_TOKEN_REQUESTS_JSON}
            disabled={!canUpdate}
            rows={3}
          />
          <SettingsTextArea
            {...field('exactMatchClaims')}
            label={OAuth2Strings.PAGE.EXACT_MATCH_CLAIMS}
            helpText={OAuth2Strings.PAGE.CLAIMS_MUST_MATCH_EXACTLY_JSON}
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
