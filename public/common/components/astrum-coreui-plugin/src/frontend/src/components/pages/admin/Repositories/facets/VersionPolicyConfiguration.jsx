
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxFormSelect} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function VersionPolicyConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  let {
    pristineData: {name},
    data: {format}
  } = currentParent.context;

  const isEdit = !!name;
  format = format === 'maven2' ? 'maven' : format;

  const versionPolicyOptions = [
    {key: 'Release', value: 'RELEASE'},
    {key: 'Snapshot', value: 'SNAPSHOT'},
    {key: 'Mixed', value: 'MIXED'}
  ];

  return (
    <NxFormGroup
      label={EDITOR.VERSION_POLICY_LABEL}
      className="as-form-group-version-policy"
      sublabel={EDITOR.VERSION_POLICY_SUBLABEL}
      isRequired
    >
      <NxFormSelect
        {...FormUtils.selectProps(`${format}.versionPolicy`, currentParent)}
        onChange={FormUtils.handleUpdate(`${format}.versionPolicy`, sendParent)}
        disabled={isEdit}
      >
        {versionPolicyOptions.map(({key, value}) => (
          <option key={key} value={value}>
            {key}
          </option>
        ))}
      </NxFormSelect>
    </NxFormGroup>
  );
}
