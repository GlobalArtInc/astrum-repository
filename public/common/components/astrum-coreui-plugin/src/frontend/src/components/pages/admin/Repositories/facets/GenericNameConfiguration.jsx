
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsFormGroup,
  AsCheckbox,
  AsTextInput,
  AsFieldset,
  AsReadOnly
} from '@globalart/astrum-react-shared-components';

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
      <h2 className="as-h2">{EDITOR.CONFIGURATION_CAPTION}</h2>
      <AsFormGroup label={EDITOR.NAME_LABEL} isRequired className="as-form-group-name">
        <AsTextInput
          {...FormUtils.fieldProps('name', currentParent)}
          onChange={FormUtils.handleUpdate('name', sendParent)}
          disabled={isEdit}
        />
      </AsFormGroup>

      {isEdit && (
        <AsReadOnly>
          <AsReadOnly.Label>{EDITOR.URL_LABEL}</AsReadOnly.Label>
          <AsReadOnly.Data>{url}</AsReadOnly.Data>
        </AsReadOnly>
      )}

      <AsFieldset label={EDITOR.STATUS_LABEL} className="as-form-group-status">
        <AsCheckbox
          {...FormUtils.checkboxProps('online', currentParent)}
          onChange={FormUtils.handleUpdate('online', sendParent)}
        >
          {EDITOR.STATUS_DESCR}
        </AsCheckbox>
      </AsFieldset>
    </>
  );
}
