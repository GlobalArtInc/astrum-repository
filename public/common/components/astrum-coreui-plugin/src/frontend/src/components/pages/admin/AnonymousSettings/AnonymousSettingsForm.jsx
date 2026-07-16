
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  FormUtils
} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxCheckbox,
  NxFormGroup,
  NxFormSelect,
  NxStatefulForm,
  NxTextInput,
  NxTooltip,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

import AnonymousMachine from './AnonymousMachine';

export default function AnonymousSettingsForm() {
  const [current, send] = useMachine(AnonymousMachine, {devTools: true});
  const {data, isPristine, realms} = current.context;

  function discard() {
    send({type: 'RESET'});
  }

  return <NxStatefulForm
      {...FormUtils.formProps(current, send)}
      additionalFooterBtns={
        <NxTooltip title={FormUtils.discardTooltip({isPristine})}>
          <NxButton type="button" className={isPristine && 'disabled'} onClick={discard}>
            {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
          </NxButton>
        </NxTooltip>
      }>
    {() => <>
      <NxFormGroup label={UIStrings.ANONYMOUS_SETTINGS.ENABLED_CHECKBOX_LABEL} isRequired>
        <NxCheckbox
            {...FormUtils.checkboxProps('enabled', current)}
            onChange={FormUtils.handleUpdate('enabled', send)}>
          {UIStrings.ANONYMOUS_SETTINGS.ENABLED_CHECKBOX_DESCRIPTION}
        </NxCheckbox>
      </NxFormGroup>
      <NxFormGroup label={UIStrings.ANONYMOUS_SETTINGS.USERNAME_TEXTFIELD_LABEL} isRequired>
        <NxTextInput
          {...FormUtils.fieldProps('userId', current)}
          onChange={FormUtils.handleUpdate('userId', send)}
          onBlur={FormUtils.trimOnBlur('userId', send)}
        />
      </NxFormGroup>
      <NxFormGroup label={UIStrings.ANONYMOUS_SETTINGS.REALM_SELECT_LABEL} isRequired>
        <NxFormSelect name='realmName' value={data.realmName} onChange={FormUtils.handleUpdate('realmName', send)} validatable>
          {realms?.map((realm) =>
              <option key={realm.id} value={realm.id}>{realm.name}</option>
          )}
        </NxFormSelect>
      </NxFormGroup>
    </>}
  </NxStatefulForm>;
}
