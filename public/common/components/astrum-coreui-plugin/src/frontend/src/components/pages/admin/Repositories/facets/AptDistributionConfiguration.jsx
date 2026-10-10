
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsTextInput, AsH2} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {
  CAPTION,
  DISTRIBUTION: {LABEL, SUBLABEL}
} = UIStrings.REPOSITORIES.EDITOR.APT;

export default function AptDistributionConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <>
      <AsH2>{CAPTION}</AsH2>
      <AsFormGroup label={LABEL} sublabel={SUBLABEL} isRequired>
        <AsTextInput
          {...FormUtils.fieldProps('apt.distribution', parentState)}
          onChange={FormUtils.handleUpdate('apt.distribution', sendParent)}
          className="as-text-input--long"
        />
      </AsFormGroup>
    </>
  );
}
