
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  NxFormGroup,
  NxCheckbox,
  NxTextInput,
  NxFieldset,
  NxReadOnly
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function GenericNameConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {
    pristineData: {name},
    data: {url}
  } = currentParent.context;
  const isEdit = !!name;

  return (
    <>
      <h2 className="nx-h2">{EDITOR.CONFIGURATION_CAPTION}</h2>
      <NxFormGroup label={EDITOR.NAME_LABEL} isRequired className="as-form-group-name">
        <NxTextInput
          {...FormUtils.fieldProps('name', currentParent)}
          onChange={FormUtils.handleUpdate('name', sendParent)}
          disabled={isEdit}
        />
      </NxFormGroup>

      {isEdit && (
        <NxReadOnly>
          <NxReadOnly.Label>{EDITOR.URL_LABEL}</NxReadOnly.Label>
          <NxReadOnly.Data>{url}</NxReadOnly.Data>
        </NxReadOnly>
      )}

      <NxFieldset label={EDITOR.STATUS_LABEL} className="as-form-group-status">
        <NxCheckbox
          {...FormUtils.checkboxProps('online', currentParent)}
          onChange={FormUtils.handleUpdate('online', sendParent)}
        >
          {EDITOR.STATUS_DESCR}
        </NxCheckbox>
      </NxFieldset>
    </>
  );
}
