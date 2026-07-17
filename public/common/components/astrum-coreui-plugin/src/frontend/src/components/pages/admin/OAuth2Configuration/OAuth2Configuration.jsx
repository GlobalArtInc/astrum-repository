
import React from 'react';
import {useMachine} from '@xstate/react';

import {FormUtils, UseAstrumTruststore} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxFieldset,
  NxFormGroup,
  NxStatefulForm,
  NxTooltip
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

import {NxTextInput} from "@globalart/react-shared-components";

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
        <NxStatefulForm
            {...FormUtils.formProps(current, send)}
            additionalFooterBtns={
              <NxTooltip title={FormUtils.discardTooltip({isPristine})}>
                <NxButton type="button" className={isPristine && 'disabled'} onClick={discard}>
                  {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                </NxButton>
              </NxTooltip>
            }>
          <NxFieldset label={LABELS.OAUTH_2_0_CONFIGURATION}>
            <div className="nx-sub-label nx-oauth2-required-advice-left">
              {LABELS.SUBLABELS.REQUIRED_FIELDS}
            </div>
          </NxFieldset>
          <NxFieldset label={LABELS.OIDC_SETTINGS}>
            {current.context.data?.redirectUri &&
              <NxFormGroup label={FIELDS.REDIRECT_URI} sublabel={LABELS.SUBLABELS.REDIRECT_URI}>
                <NxTextInput
                    className="nx-text-input--long"
                    value={current.context.data.redirectUri}
                    isPristine
                    readOnly
                    onChange={() => {}}
                />
              </NxFormGroup>
            }
            <NxFormGroup label={FIELDS.ISSUER_URL} sublabel={LABELS.SUBLABELS.ISSUER_URL}>
              <NxTextInput
                  className="nx-text-input--long"
                  {...FormUtils.fieldProps('issuerUrl', current)}
                  onChange={FormUtils.handleUpdate('issuerUrl', send)}
                  onBlur={FormUtils.trimOnBlur('issuerUrl', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.CLIENT_ID} sublabel={LABELS.SUBLABELS.CLIENT_ID} isRequired>
              <NxTextInput
                  type="password"
                  {...FormUtils.fieldProps('clientId', current)}
                  onChange={FormUtils.handleUpdate('clientId', send)}
                  onBlur={FormUtils.trimOnBlur('clientId', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.CLIENT_SECRET} sublabel={LABELS.SUBLABELS.CLIENT_SECRET}>
              <NxTextInput
                  type="password"
                  autoComplete="new-password"
                  {...FormUtils.fieldProps('clientSecret', current)}
                  onChange={FormUtils.handleUpdate('clientSecret', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.IDP_AUTHORIZATION_URL} sublabel={LABELS.SUBLABELS.IDP_AUTHORIZATION_URL}>
              <NxTextInput
                  className="nx-text-input--long"
                  {...FormUtils.fieldProps('idpAuthorizationUrl', current)}
                  onChange={FormUtils.handleUpdate('idpAuthorizationUrl', send)}
                  onBlur={FormUtils.trimOnBlur('idpAuthorizationUrl', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.IDP_LOGOUT_URL} sublabel={LABELS.SUBLABELS.IDP_LOGOUT_URL}>
              <NxTextInput
                  className="nx-text-input--long"
                  {...FormUtils.fieldProps('idpLogoutUrl', current)}
                  onChange={FormUtils.handleUpdate('idpLogoutUrl', send)}
                  onBlur={FormUtils.trimOnBlur('idpLogoutUrl', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.IDP_TOKEN_URL} sublabel={LABELS.SUBLABELS.IDP_TOKEN_URL}>
              <NxTextInput
                  className="nx-text-input--long"
                  {...FormUtils.fieldProps('idpTokenUrl', current)}
                  onChange={FormUtils.handleUpdate('idpTokenUrl', send)}
                  onBlur={FormUtils.trimOnBlur('idpTokenUrl', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.IDP_JWKS_URL} sublabel={LABELS.SUBLABELS.IDP_JWKS_URL}>
              <NxTextInput
                  className="nx-text-input--long"
                  {...FormUtils.fieldProps('idpJwksUrl', current)}
                  onChange={FormUtils.handleUpdate('idpJwksUrl', send)}
                  onBlur={FormUtils.trimOnBlur('idpJwksUrl', send)}
              />
            </NxFormGroup>
            <UseAstrumTruststore
                remoteUrl={current.context.data?.idpTokenUrl}
                {...FormUtils.checkboxProps('useTrustStore', current)}
                onChange={FormUtils.handleUpdate('useTrustStore', send)}
            />
          </NxFieldset>

          <NxFieldset label={LABELS.CLAIM_MAPPINGS}>
            <NxFormGroup label={FIELDS.USERNAME_CLAIM} sublabel={LABELS.SUBLABELS.USERNAME_CLAIM}>
              <NxTextInput
                  {...FormUtils.fieldProps('usernameClaim', current)}
                  onChange={FormUtils.handleUpdate('usernameClaim', send)}
                  onBlur={FormUtils.trimOnBlur('usernameClaim', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.FIRST_NAME_CLAIM} sublabel={LABELS.SUBLABELS.FIRST_NAME_CLAIM}>
              <NxTextInput
                  {...FormUtils.fieldProps('firstNameClaim', current)}
                  onChange={FormUtils.handleUpdate('firstNameClaim', send)}
                  onBlur={FormUtils.trimOnBlur('firstNameClaim', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.LAST_NAME_CLAIM} sublabel={LABELS.SUBLABELS.LAST_NAME_CLAIM}>
              <NxTextInput
                  {...FormUtils.fieldProps('lastNameClaim', current)}
                  onChange={FormUtils.handleUpdate('lastNameClaim', send)}
                  onBlur={FormUtils.trimOnBlur('lastNameClaim', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.EMAIL_CLAIM} sublabel={LABELS.SUBLABELS.EMAIL_CLAIM}>
              <NxTextInput
                  {...FormUtils.fieldProps('emailClaim', current)}
                  onChange={FormUtils.handleUpdate('emailClaim', send)}
                  onBlur={FormUtils.trimOnBlur('emailClaim', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.GROUPS_CLAIM} sublabel={LABELS.SUBLABELS.GROUPS_CLAIM}>
              <NxTextInput
                  {...FormUtils.fieldProps('groupsClaim', current)}
                  onChange={FormUtils.handleUpdate('groupsClaim', send)}
                  onBlur={FormUtils.trimOnBlur('groupsClaim', send)}
              />
            </NxFormGroup>
          </NxFieldset>

          <NxFieldset label={LABELS.JWT_SETTINGS}>
            <NxFormGroup label={FIELDS.IDP_JWS_ALGORITHM} sublabel={LABELS.SUBLABELS.IDP_JWS_ALGORITHM}>
              <NxTextInput
                {...FormUtils.fieldProps('idpJwsAlgorithm', current)}
                onChange={FormUtils.handleUpdate('idpJwsAlgorithm', send)}
                onBlur={FormUtils.trimOnBlur('idpJwsAlgorithm', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.IDP_JWKS} sublabel={LABELS.SUBLABELS.IDP_JWKS}>
              <NxTextInput
                type="textarea"
                className="nx-text-input--long nx-oauth2-text-input-medium"
                {...FormUtils.fieldProps('idpJwks', current)}
                onChange={FormUtils.handleUpdate('idpJwks', send)}
                placeholder={LABELS.OPTIONAL_JWKS}
              />
            </NxFormGroup>
          </NxFieldset>

          <NxFieldset label={LABELS.ADVANCED_SETTINGS}>
            <NxFormGroup label={FIELDS.AUTHORIZATION_CUSTOM_PARAMS} sublabel={LABELS.SUBLABELS.AUTHORIZATION_CUSTOM_PARAMS}>
              <NxTextInput
                type="textarea"
                className="nx-text-input--long nx-oauth2-text-input-medium"
                {...FormUtils.fieldProps('authorizationCustomParams', current)}
                onChange={FormUtils.handleUpdate('authorizationCustomParams', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.TOKEN_REQUEST_CUSTOM_PARAMS} sublabel={LABELS.SUBLABELS.TOKEN_REQUEST_CUSTOM_PARAMS}>
              <NxTextInput
                type="textarea"
                className="nx-text-input--long nx-oauth2-text-input-medium"
                {...FormUtils.fieldProps('tokenRequestCustomParams', current)}
                onChange={FormUtils.handleUpdate('tokenRequestCustomParams', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.EXACT_MATCH_CLAIMS} sublabel={LABELS.SUBLABELS.EXACT_MATCH_CLAIMS}>
              <NxTextInput
                  type="textarea"
                  className="nx-text-input--long nx-oauth2-text-input-medium"
                  {...FormUtils.fieldProps('exactMatchClaims', current)}
                  onChange={FormUtils.handleUpdate('exactMatchClaims', send)}
              />
            </NxFormGroup>
          </NxFieldset>
        </NxStatefulForm>
      </Section>
    </ContentBody>
  </Page>;
}
