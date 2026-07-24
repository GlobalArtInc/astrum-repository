
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxFormSelect} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function LayoutPolicyConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  let {format} = currentParent.context.data;
  format = format === 'maven2' ? 'maven' : format;

  const layoutPolicyOptions = [
    {key: 'Strict', value: 'STRICT'},
    {key: 'Permissive', value: 'PERMISSIVE'}
  ];

  return (
    <NxFormGroup
      label={EDITOR.LAYOUT_POLICY_LABEL}
      className="as-form-group-layout-policy"
      sublabel={EDITOR.LAYOUT_POLICY_SUBLABEL}
      isRequired
    >
      <NxFormSelect
        {...FormUtils.selectProps(`${format}.layoutPolicy`, currentParent)}
        onChange={FormUtils.handleUpdate(`${format}.layoutPolicy`, sendParent)}
      >
        {layoutPolicyOptions.map(({key, value}) => (
          <option key={key} value={value}>
            {key}
          </option>
        ))}
      </NxFormSelect>
    </NxFormGroup>
  );
}
