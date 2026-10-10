
import React from 'react';
import {useMachine} from '@xstate/react';
import {faIdCard} from '@fortawesome/free-solid-svg-icons';

import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFieldset,
  AsFormGroup,
  AsFormSelect,
  AsStatefulForm,
  AsTooltip
} from '@globalart/astrum-react-shared-components';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import SamlConfigurationMachine from './SamlConfigurationMachine';
import UIStrings from '../../../../constants/UIStrings';

import './SamlConfiguration.scss';
import {AsTextInput} from "@globalart/astrum-react-shared-components";

const {FIELDS, LABELS} = UIStrings.SAML_CONFIGURATION;

export default function SamlConfiguration() {
  const [current, send] = useMachine(SamlConfigurationMachine, {devTools: true});

  const {data: settings, isPristine} = current.context;

  function discard() {
    send({type: 'RESET'});
  }

  return <Page>
    <PageHeader>
      <PageTitle
        icon={UIStrings.SAML_CONFIGURATION.MENU.icon}
        text={UIStrings.SAML_CONFIGURATION.MENU.text}
        description={UIStrings.SAML_CONFIGURATION.MENU.description}
      />
    </PageHeader>
    <ContentBody className='as-saml-configuration'>
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
          <AsFormGroup label={FIELDS.idpMetadataLabel} isRequired>
            <AsTextInput
                type="textarea"
                className="as-text-input--long"
                {...FormUtils.fieldProps('idpMetadata', current)}
                onChange={FormUtils.handleUpdate('idpMetadata', send)}/>
          </AsFormGroup>
          <AsFormGroup label={FIELDS.entityIdUriLabel} sublabel={FIELDS.entityIdUriDescription} isRequired>
            <AsTextInput
                className="as-text-input--long"
                {...FormUtils.fieldProps('entityIdUri', current)}
                onChange={FormUtils.handleUpdate('entityIdUri', send)}/>
          </AsFormGroup>
          <AsFormGroup
              label={FIELDS.validateResponseSignatureLabel}
              sublabel={FIELDS.validateResponseSignatureDescription}>
            <AsFormSelect
                name="validateResponseSignature"
                value={settings.validateResponseSignature}
                onChange={FormUtils.handleUpdate('validateResponseSignature', send)}>
              <option value="default">{UIStrings.PAGES.SAML_SIGNATURE.DEFAULT}</option>
              <option value="true">{UIStrings.PAGES.SAML_SIGNATURE.TRUE}</option>
              <option value="false">{UIStrings.PAGES.SAML_SIGNATURE.FALSE}</option>
            </AsFormSelect>
          </AsFormGroup>
          <AsFormGroup
              label={FIELDS.validateAssertionSignatureLabel}
              sublabel={FIELDS.validateAssertionSignatureDescription}>
            <AsFormSelect
                name="validateAssertionSignature"
                value={settings.validateAssertionSignature}
                onChange={FormUtils.handleUpdate('validateAssertionSignature', send)}>
              <option value="default">{UIStrings.PAGES.SAML_SIGNATURE.DEFAULT}</option>
              <option value="true">{UIStrings.PAGES.SAML_SIGNATURE.TRUE}</option>
              <option value="false">{UIStrings.PAGES.SAML_SIGNATURE.FALSE}</option>
            </AsFormSelect>
          </AsFormGroup>
          <AsFieldset label={LABELS.FIELDS} isRequired>
            <AsFormGroup label={FIELDS.usernameAttrLabel} isRequired>
              <AsTextInput
                {...FormUtils.fieldProps('usernameAttr', current)}
                onChange={FormUtils.handleUpdate('usernameAttr', send)}
                onBlur={FormUtils.trimOnBlur('usernameAttr', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.firstNameAttrLabel}>
              <AsTextInput
                {...FormUtils.fieldProps('firstNameAttr', current)}
                onChange={FormUtils.handleUpdate('firstNameAttr', send)}
                onBlur={FormUtils.trimOnBlur('firstNameAttr', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.lastNameAttrLabel}>
              <AsTextInput
                {...FormUtils.fieldProps('lastNameAttr', current)}
                onChange={FormUtils.handleUpdate('lastNameAttr', send)}
                onBlur={FormUtils.trimOnBlur('lastNameAttr', send)} 
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.emailAttrLabel}>
              <AsTextInput
                {...FormUtils.fieldProps('emailAttr', current)}
                onChange={FormUtils.handleUpdate('emailAttr', send)}
                onBlur={FormUtils.trimOnBlur('emailAttr', send)} 
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.roleAttrLabel}>
              <AsTextInput
                {...FormUtils.fieldProps('roleAttr', current)}
                onChange={FormUtils.handleUpdate('roleAttr', send)}
                onBlur={FormUtils.trimOnBlur('roleAttr', send)} 
              />
            </AsFormGroup>
          </AsFieldset>
        </AsStatefulForm>
      </Section>
    </ContentBody>
  </Page>;
}
