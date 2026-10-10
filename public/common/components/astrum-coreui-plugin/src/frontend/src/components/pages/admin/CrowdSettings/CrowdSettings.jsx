 
import React from 'react';
import {useMachine, useActor} from '@xstate/react';
import {faTrash} from "@fortawesome/free-solid-svg-icons";
import {faAtlassian} from '@fortawesome/free-brands-svg-icons';
import CrowdSettingsMachine from './CrowdSettingsMachine';
import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsCheckbox,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsStatefulForm,
  AsTextInput,
  AsTooltip,
} from '@globalart/react-shared-components';

 import {
   ContentBody,
   Page,
   PageHeader,
   PageTitle,
   Section
 } from '@globalart/astrum-ui-plugin';

 import UIStrings from '../../../../constants/UIStrings';

export default function CrowdSettings() {
  const service = useMachine(CrowdSettingsMachine, {devTools: true})[2];
  return <CrowdSettingsForm service={service}/>;
}

export function CrowdSettingsForm({service}) {
  const [current, send] = useActor(service);

  const {data, isPristine, validationErrors} = current.context;
  const isInvalid = FormUtils.isInvalid(validationErrors);

  function discard() {
    send({type: 'RESET'});
  }

  function verifyConnection() {
    send({type: 'VERIFY-CONNECTION'});
  }

  function clearCache() {
    send({type: 'CLEAR-CACHE'});
  }

  return <Page>
    <PageHeader><PageTitle
        icon={UIStrings.CROWD_SETTINGS.MENU.icon}
        text={UIStrings.CROWD_SETTINGS.MENU.text}
        description={UIStrings.CROWD_SETTINGS.MENU.description}
    /></PageHeader>
    <ContentBody className='as-crowd-settings'>
      <Section>
        <AsStatefulForm
            {...FormUtils.formProps(current, send)}
            additionalFooterBtns={<>
              <AsTooltip title={FormUtils.discardTooltip({isPristine})}>
                <AsButton type="button" className={(isPristine) && 'disabled'} onClick={discard}>
                  {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                </AsButton>
              </AsTooltip>
              <AsButton type="button" disabled={isInvalid} onClick={verifyConnection}>
                {UIStrings.CROWD_SETTINGS.BUTTONS.VERIFY_BUTTON_LABEL}
              </AsButton>
              <AsButton type="button" onClick={clearCache}>
                <AsFontAwesomeIcon icon={faTrash}/>
                <span>{UIStrings.CROWD_SETTINGS.BUTTONS.CLEAR_BUTTON_LABEL}</span>
              </AsButton>
            </>}>
          <AsFormGroup label={UIStrings.CROWD_SETTINGS.FIELDS.enabledLabel}>
            <AsCheckbox
                {...FormUtils.checkboxProps('enabled', current)}
                onChange={FormUtils.handleUpdate('enabled', send)}>
              {UIStrings.CROWD_SETTINGS.FIELDS.enabledDescription}
            </AsCheckbox>
          </AsFormGroup>
          <AsFormGroup label={UIStrings.CROWD_SETTINGS.FIELDS.realmActiveLabel}>
            <AsCheckbox
                {...FormUtils.checkboxProps('realmActive', current)}
                onChange={FormUtils.handleUpdate('realmActive', send)}>
              <span dangerouslySetInnerHTML={UIStrings.CROWD_SETTINGS.FIELDS.realmActiveDescription}/>
            </AsCheckbox>
          </AsFormGroup>
          <AsFormGroup
              label={UIStrings.CROWD_SETTINGS.FIELDS.urlLabel}
              sublabel={UIStrings.CROWD_SETTINGS.FIELDS.urlDescription}
              isRequired>
            <AsTextInput
                className="as-text-input--long"
                {...FormUtils.fieldProps('url', current)}
                onChange={FormUtils.handleUpdate('url', send)}/>
          </AsFormGroup>
          {data.url?.startsWith('https') && <>
            <AsFormGroup
                label={UIStrings.CROWD_SETTINGS.FIELDS.useTrustStoreLabel}
                sublabel={<span dangerouslySetInnerHTML={UIStrings.CROWD_SETTINGS.FIELDS.useTrustStoreDescription}/>}
                isRequired>
              <AsCheckbox
                  {...FormUtils.checkboxProps('useTrustStoreForUrl', current)}
                  onChange={FormUtils.handleUpdate('useTrustStoreForUrl', send)}>
                {UIStrings.CROWD_SETTINGS.FIELDS.useTrustStoreText}
              </AsCheckbox>
            </AsFormGroup>
          </>}
          <AsFormGroup label={UIStrings.CROWD_SETTINGS.FIELDS.applicationNameLabel} isRequired>
            <AsTextInput
                autoComplete="off"
                {...FormUtils.fieldProps('applicationName', current)}
                onChange={FormUtils.handleUpdate('applicationName', send)}/>
          </AsFormGroup>
          <AsFormGroup label={UIStrings.CROWD_SETTINGS.FIELDS.applicationPasswordLabel} isRequired>
            <AsTextInput
                type="password"
                autoComplete="new-password"
                {...FormUtils.fieldProps('applicationPassword', current)}
                onChange={FormUtils.handleUpdate('applicationPassword', send)}/>
          </AsFormGroup>
          <AsFormGroup
              label={UIStrings.CROWD_SETTINGS.FIELDS.timeoutLabel}
              sublabel={UIStrings.CROWD_SETTINGS.FIELDS.timeoutDescription}>
            <AsTextInput
                {...FormUtils.fieldProps('timeout', current)}
                onChange={FormUtils.handleUpdate('timeout', send)}/>
          </AsFormGroup>
        </AsStatefulForm>
      </Section>
    </ContentBody>
  </Page>;
}
