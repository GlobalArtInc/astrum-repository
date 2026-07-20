
import React from 'react';
import {useMachine} from '@xstate/react';
import {faIdCard} from '@fortawesome/free-solid-svg-icons';

import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxFieldset,
  NxFormGroup,
  NxFormSelect,
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

import SamlConfigurationMachine from './SamlConfigurationMachine';
import UIStrings from '../../../../constants/UIStrings';

import './SamlConfiguration.scss';
import {NxTextInput} from "@globalart/react-shared-components";

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
        <NxStatefulForm
            {...FormUtils.formProps(current, send)}
            additionalFooterBtns={
              <NxTooltip title={FormUtils.discardTooltip({isPristine})}>
                <NxButton type="button" className={isPristine && 'disabled'} onClick={discard}>
                  {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                </NxButton>
              </NxTooltip>
            }>
          <NxFormGroup label={FIELDS.idpMetadataLabel} isRequired>
            <NxTextInput
                type="textarea"
                className="nx-text-input--long"
                {...FormUtils.fieldProps('idpMetadata', current)}
                onChange={FormUtils.handleUpdate('idpMetadata', send)}/>
          </NxFormGroup>
          <NxFormGroup label={FIELDS.entityIdUriLabel} sublabel={FIELDS.entityIdUriDescription} isRequired>
            <NxTextInput
                className="nx-text-input--long"
                {...FormUtils.fieldProps('entityIdUri', current)}
                onChange={FormUtils.handleUpdate('entityIdUri', send)}/>
          </NxFormGroup>
          <NxFormGroup
              label={FIELDS.validateResponseSignatureLabel}
              sublabel={FIELDS.validateResponseSignatureDescription}>
            <NxFormSelect
                name="validateResponseSignature"
                value={settings.validateResponseSignature}
                onChange={FormUtils.handleUpdate('validateResponseSignature', send)}>
              <option value="default">Default</option>
              <option value="true">True</option>
              <option value="false">False</option>
            </NxFormSelect>
          </NxFormGroup>
          <NxFormGroup
              label={FIELDS.validateAssertionSignatureLabel}
              sublabel={FIELDS.validateAssertionSignatureDescription}>
            <NxFormSelect
                name="validateAssertionSignature"
                value={settings.validateAssertionSignature}
                onChange={FormUtils.handleUpdate('validateAssertionSignature', send)}>
              <option value="default">Default</option>
              <option value="true">True</option>
              <option value="false">False</option>
            </NxFormSelect>
          </NxFormGroup>
          <NxFieldset label={LABELS.FIELDS} isRequired>
            <NxFormGroup label={FIELDS.usernameAttrLabel} isRequired>
              <NxTextInput
                {...FormUtils.fieldProps('usernameAttr', current)}
                onChange={FormUtils.handleUpdate('usernameAttr', send)}
                onBlur={FormUtils.trimOnBlur('usernameAttr', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.firstNameAttrLabel}>
              <NxTextInput
                {...FormUtils.fieldProps('firstNameAttr', current)}
                onChange={FormUtils.handleUpdate('firstNameAttr', send)}
                onBlur={FormUtils.trimOnBlur('firstNameAttr', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.lastNameAttrLabel}>
              <NxTextInput
                {...FormUtils.fieldProps('lastNameAttr', current)}
                onChange={FormUtils.handleUpdate('lastNameAttr', send)}
                onBlur={FormUtils.trimOnBlur('lastNameAttr', send)} 
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.emailAttrLabel}>
              <NxTextInput
                {...FormUtils.fieldProps('emailAttr', current)}
                onChange={FormUtils.handleUpdate('emailAttr', send)}
                onBlur={FormUtils.trimOnBlur('emailAttr', send)} 
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.roleAttrLabel}>
              <NxTextInput
                {...FormUtils.fieldProps('roleAttr', current)}
                onChange={FormUtils.handleUpdate('roleAttr', send)}
                onBlur={FormUtils.trimOnBlur('roleAttr', send)} 
              />
            </NxFormGroup>
          </NxFieldset>
        </NxStatefulForm>
      </Section>
    </ContentBody>
  </Page>;
}
