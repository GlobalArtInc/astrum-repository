
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFieldset, AsCheckbox} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {LABEL, DESCR} = UIStrings.REPOSITORIES.EDITOR.APT.FLAT;

export default function AptFlatConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <AsFieldset label={LABEL} isRequired>
      <AsCheckbox
        {...FormUtils.checkboxProps('apt.flat', parentState)}
        onChange={FormUtils.handleUpdate('apt.flat', sendParent)}
      >
        {DESCR}
      </AsCheckbox>
    </AsFieldset>
  );
}
