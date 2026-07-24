
import React from 'react';

import {NxH2, NxCheckbox, NxFieldset} from '@globalart/react-shared-components';
import {FormUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function RegistryApiSupportConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  return (
    <>
      <NxH2>{EDITOR.REGISTRY_API_SUPPORT_CAPTION}</NxH2>
      <NxFieldset
        label={EDITOR.REGISTRY_API_SUPPORT_LABEL}
        className="as-form-group-registry-api-support"
      >
        <NxCheckbox
          {...FormUtils.checkboxProps('docker.v1Enabled', currentParent)}
          onChange={FormUtils.handleUpdate('docker.v1Enabled', sendParent)}
        >
          {EDITOR.REGISTRY_API_SUPPORT_DESCR}
        </NxCheckbox>
      </NxFieldset>
    </>
  );
}
