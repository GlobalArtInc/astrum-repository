
import React from 'react';

import {AsH2, AsCheckbox, AsFieldset} from '@globalart/astrum-react-shared-components';
import {FormUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function RegistryApiSupportConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  return (
    <>
      <AsH2>{EDITOR.REGISTRY_API_SUPPORT_CAPTION}</AsH2>
      <AsFieldset
        label={EDITOR.REGISTRY_API_SUPPORT_LABEL}
        className="as-form-group-registry-api-support"
      >
        <AsCheckbox
          {...FormUtils.checkboxProps('docker.v1Enabled', currentParent)}
          onChange={FormUtils.handleUpdate('docker.v1Enabled', sendParent)}
        >
          {EDITOR.REGISTRY_API_SUPPORT_DESCR}
        </AsCheckbox>
      </AsFieldset>
    </>
  );
}
