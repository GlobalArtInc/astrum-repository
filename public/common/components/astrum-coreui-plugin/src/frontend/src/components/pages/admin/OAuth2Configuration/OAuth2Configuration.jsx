
import React from 'react';
import {useMachine} from '@xstate/react';

import {FormUtils, UseAstrumTruststore} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFieldset,
  AsFormGroup,
  AsStatefulForm,
  AsTooltip
} from '@globalart/react-shared-components';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import OAuth2ConfigurationMachine from './OAuth2ConfigurationMachine';
import UIStrings from '../../../../constants/UIStrings';
import './OAuth2Configuration.scss';

import {AsTextInput} from "@globalart/react-shared-components";

const {FIELDS, LABELS} = UIStrings.OAUTH2_CONFIGURATION;

export default function OAuth2Configuration() {
  const [current, send] = useMachine(OAuth2ConfigurationMachine, {devTools: true});

  const {isPristine} = current.context;

  function discard() {
    send({type: 'RESET'});
  }

  return <Page>
    <PageHeader>
      <PageTitle
        icon={UIStrings.OAUTH2_CONFIGURATION.MENU.icon}
        text={UIStrings.OAUTH2_CONFIGURATION.MENU.text}
        description={UIStrings.OAUTH2_CONFIGURATION.MENU.description}
      />
    </PageHeader>
    <ContentBody className='as-oauth2-configuration'>
      <Section>
        <AsStatefulForm
            {...FormUtils.formProps(current, send)}
            additionalFooterBtns={
              <AsTooltip title={FormUtils.discardTooltip({isPristine})}>
                <AsButton type="button" className={isPristine && 'disabled'} onClick={discard}>
                  {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                </AsButton>
              </AsTooltip>
            }>
          <AsFieldset label={LABELS.OAUTH_2_0_CONFIGURATION}>
            <div className="as-sub-label as-oauth2-required-advice-left">
              {LABELS.SUBLABELS.REQUIRED_FIELDS}
            </div>
          </AsFieldset>
          <AsFieldset label={LABELS.OIDC_SETTINGS}>
            {current.context.data?.redirectUri &&
              <AsFormGroup label={FIELDS.REDIRECT_URI} sublabel={LABELS.SUBLABELS.REDIRECT_URI}>
                <AsTextInput
                    className="as-text-input--long"
                    value={current.context.data.redirectUri}
                    isPristine
                    readOnly
                    onChange={() => {}}
                />
              </AsFormGroup>
            }
            <AsFormGroup label={FIELDS.ISSUER_URL} sublabel={LABELS.SUBLABELS.ISSUER_URL}>
              <AsTextInput
                  className="as-text-input--long"
                  {...FormUtils.fieldProps('issuerUrl', current)}
                  onChange={FormUtils.handleUpdate('issuerUrl', send)}
                  onBlur={FormUtils.trimOnBlur('issuerUrl', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.CLIENT_ID} sublabel={LABELS.SUBLABELS.CLIENT_ID} isRequired>
              <AsTextInput
                  type="password"
                  {...FormUtils.fieldProps('clientId', current)}
                  onChange={FormUtils.handleUpdate('clientId', send)}
                  onBlur={FormUtils.trimOnBlur('clientId', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.CLIENT_SECRET} sublabel={LABELS.SUBLABELS.CLIENT_SECRET}>
              <AsTextInput
                  type="password"
                  autoComplete="new-password"
                  {...FormUtils.fieldProps('clientSecret', current)}
                  onChange={FormUtils.handleUpdate('clientSecret', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.IDP_AUTHORIZATION_URL} sublabel={LABELS.SUBLABELS.IDP_AUTHORIZATION_URL}>
              <AsTextInput
                  className="as-text-input--long"
                  {...FormUtils.fieldProps('idpAuthorizationUrl', current)}
                  onChange={FormUtils.handleUpdate('idpAuthorizationUrl', send)}
                  onBlur={FormUtils.trimOnBlur('idpAuthorizationUrl', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.IDP_LOGOUT_URL} sublabel={LABELS.SUBLABELS.IDP_LOGOUT_URL}>
              <AsTextInput
                  className="as-text-input--long"
                  {...FormUtils.fieldProps('idpLogoutUrl', current)}
                  onChange={FormUtils.handleUpdate('idpLogoutUrl', send)}
                  onBlur={FormUtils.trimOnBlur('idpLogoutUrl', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.IDP_TOKEN_URL} sublabel={LABELS.SUBLABELS.IDP_TOKEN_URL}>
              <AsTextInput
                  className="as-text-input--long"
                  {...FormUtils.fieldProps('idpTokenUrl', current)}
                  onChange={FormUtils.handleUpdate('idpTokenUrl', send)}
                  onBlur={FormUtils.trimOnBlur('idpTokenUrl', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.IDP_JWKS_URL} sublabel={LABELS.SUBLABELS.IDP_JWKS_URL}>
              <AsTextInput
                  className="as-text-input--long"
                  {...FormUtils.fieldProps('idpJwksUrl', current)}
                  onChange={FormUtils.handleUpdate('idpJwksUrl', send)}
                  onBlur={FormUtils.trimOnBlur('idpJwksUrl', send)}
              />
            </AsFormGroup>
            <UseAstrumTruststore
                remoteUrl={current.context.data?.idpTokenUrl}
                {...FormUtils.checkboxProps('useTrustStore', current)}
                onChange={FormUtils.handleUpdate('useTrustStore', send)}
            />
          </AsFieldset>

          <AsFieldset label={LABELS.CLAIM_MAPPINGS}>
            <AsFormGroup label={FIELDS.USERNAME_CLAIM} sublabel={LABELS.SUBLABELS.USERNAME_CLAIM}>
              <AsTextInput
                  {...FormUtils.fieldProps('usernameClaim', current)}
                  onChange={FormUtils.handleUpdate('usernameClaim', send)}
                  onBlur={FormUtils.trimOnBlur('usernameClaim', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.FIRST_NAME_CLAIM} sublabel={LABELS.SUBLABELS.FIRST_NAME_CLAIM}>
              <AsTextInput
                  {...FormUtils.fieldProps('firstNameClaim', current)}
                  onChange={FormUtils.handleUpdate('firstNameClaim', send)}
                  onBlur={FormUtils.trimOnBlur('firstNameClaim', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.LAST_NAME_CLAIM} sublabel={LABELS.SUBLABELS.LAST_NAME_CLAIM}>
              <AsTextInput
                  {...FormUtils.fieldProps('lastNameClaim', current)}
                  onChange={FormUtils.handleUpdate('lastNameClaim', send)}
                  onBlur={FormUtils.trimOnBlur('lastNameClaim', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.EMAIL_CLAIM} sublabel={LABELS.SUBLABELS.EMAIL_CLAIM}>
              <AsTextInput
                  {...FormUtils.fieldProps('emailClaim', current)}
                  onChange={FormUtils.handleUpdate('emailClaim', send)}
                  onBlur={FormUtils.trimOnBlur('emailClaim', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.GROUPS_CLAIM} sublabel={LABELS.SUBLABELS.GROUPS_CLAIM}>
              <AsTextInput
                  {...FormUtils.fieldProps('groupsClaim', current)}
                  onChange={FormUtils.handleUpdate('groupsClaim', send)}
                  onBlur={FormUtils.trimOnBlur('groupsClaim', send)}
              />
            </AsFormGroup>
          </AsFieldset>

          <AsFieldset label={LABELS.JWT_SETTINGS}>
            <AsFormGroup label={FIELDS.IDP_JWS_ALGORITHM} sublabel={LABELS.SUBLABELS.IDP_JWS_ALGORITHM}>
              <AsTextInput
                {...FormUtils.fieldProps('idpJwsAlgorithm', current)}
                onChange={FormUtils.handleUpdate('idpJwsAlgorithm', send)}
                onBlur={FormUtils.trimOnBlur('idpJwsAlgorithm', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.IDP_JWKS} sublabel={LABELS.SUBLABELS.IDP_JWKS}>
              <AsTextInput
                type="textarea"
                className="as-text-input--long as-oauth2-text-input-medium"
                {...FormUtils.fieldProps('idpJwks', current)}
                onChange={FormUtils.handleUpdate('idpJwks', send)}
                placeholder={LABELS.OPTIONAL_JWKS}
              />
            </AsFormGroup>
          </AsFieldset>

          <AsFieldset label={LABELS.ADVANCED_SETTINGS}>
            <AsFormGroup label={FIELDS.AUTHORIZATION_CUSTOM_PARAMS} sublabel={LABELS.SUBLABELS.AUTHORIZATION_CUSTOM_PARAMS}>
              <AsTextInput
                type="textarea"
                className="as-text-input--long as-oauth2-text-input-medium"
                {...FormUtils.fieldProps('authorizationCustomParams', current)}
                onChange={FormUtils.handleUpdate('authorizationCustomParams', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.TOKEN_REQUEST_CUSTOM_PARAMS} sublabel={LABELS.SUBLABELS.TOKEN_REQUEST_CUSTOM_PARAMS}>
              <AsTextInput
                type="textarea"
                className="as-text-input--long as-oauth2-text-input-medium"
                {...FormUtils.fieldProps('tokenRequestCustomParams', current)}
                onChange={FormUtils.handleUpdate('tokenRequestCustomParams', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.EXACT_MATCH_CLAIMS} sublabel={LABELS.SUBLABELS.EXACT_MATCH_CLAIMS}>
              <AsTextInput
                  type="textarea"
                  className="as-text-input--long as-oauth2-text-input-medium"
                  {...FormUtils.fieldProps('exactMatchClaims', current)}
                  onChange={FormUtils.handleUpdate('exactMatchClaims', send)}
              />
            </AsFormGroup>
          </AsFieldset>
        </AsStatefulForm>
      </Section>
    </ContentBody>
  </Page>;
}
