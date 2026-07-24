
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxFormSelect} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function RpmDeployPolicyConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  let {format} = currentParent.context.data;

  const deployPolicyOptions = [
    {key: 'Strict', value: 'STRICT'},
    {key: 'Permissive', value: 'PERMISSIVE'}
  ];

  return (
    <NxFormGroup
      label={EDITOR.LAYOUT_POLICY_LABEL}
      className="as-form-group-deploy-policy"
      sublabel={EDITOR.DEPLOY_POLICY_SUBLABEL}
      isRequired
    >
      <NxFormSelect
        {...FormUtils.selectProps(`${format}.deployPolicy`, currentParent)}
        onChange={FormUtils.handleUpdate(`${format}.deployPolicy`, sendParent)}
      >
        {deployPolicyOptions.map(({key, value}) => (
          <option key={key} value={value}>
            {key}
          </option>
        ))}
      </NxFormSelect>
    </NxFormGroup>
  );
}
