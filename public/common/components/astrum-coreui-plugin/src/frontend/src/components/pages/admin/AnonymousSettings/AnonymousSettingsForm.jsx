
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  FormUtils
} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsCheckbox,
  AsFormGroup,
  AsFormSelect,
  AsStatefulForm,
  AsTextInput,
  AsTooltip,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

import AnonymousMachine from './AnonymousMachine';

export default function AnonymousSettingsForm() {
  const [current, send] = useMachine(AnonymousMachine, {devTools: true});
  const {data, isPristine, realms} = current.context;

  function discard() {
    send({type: 'RESET'});
  }

  return <AsStatefulForm
      {...FormUtils.formProps(current, send)}
      additionalFooterBtns={
        <AsTooltip title={FormUtils.discardTooltip({isPristine})}>
          <AsButton type="button" className={isPristine && 'disabled'} onClick={discard}>
            {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
          </AsButton>
        </AsTooltip>
      }>
    {() => <>
      <AsFormGroup label={UIStrings.ANONYMOUS_SETTINGS.ENABLED_CHECKBOX_LABEL} isRequired>
        <AsCheckbox
            {...FormUtils.checkboxProps('enabled', current)}
            onChange={FormUtils.handleUpdate('enabled', send)}>
          {UIStrings.ANONYMOUS_SETTINGS.ENABLED_CHECKBOX_DESCRIPTION}
        </AsCheckbox>
      </AsFormGroup>
      <AsFormGroup label={UIStrings.ANONYMOUS_SETTINGS.USERNAME_TEXTFIELD_LABEL} isRequired>
        <AsTextInput
          {...FormUtils.fieldProps('userId', current)}
          onChange={FormUtils.handleUpdate('userId', send)}
          onBlur={FormUtils.trimOnBlur('userId', send)}
        />
      </AsFormGroup>
      <AsFormGroup label={UIStrings.ANONYMOUS_SETTINGS.REALM_SELECT_LABEL} isRequired>
        <AsFormSelect name='realmName' value={data.realmName} onChange={FormUtils.handleUpdate('realmName', send)} validatable>
          {realms?.map((realm) =>
              <option key={realm.id} value={realm.id}>{realm.name}</option>
          )}
        </AsFormSelect>
      </AsFormGroup>
    </>}
  </AsStatefulForm>;
}
