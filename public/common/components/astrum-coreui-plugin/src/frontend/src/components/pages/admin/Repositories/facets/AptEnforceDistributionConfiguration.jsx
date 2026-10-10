
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFieldset, AsCheckbox} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {LABEL, DESCR} = UIStrings.REPOSITORIES.EDITOR.APT.ENFORCE_DISTRIBUTION;

export default function AptEnforceDistributionConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <AsFieldset label={LABEL} isRequired>
      <AsCheckbox
        {...FormUtils.checkboxProps('apt.enforceDistribution', parentState)}
        onChange={FormUtils.handleUpdate('apt.enforceDistribution', sendParent)}
      >
        {DESCR}
      </AsCheckbox>
    </AsFieldset>
  );
}
