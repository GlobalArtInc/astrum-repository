
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxTextInput, NxH2} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {
  CAPTION,
  DISTRIBUTION: {LABEL, SUBLABEL}
} = UIStrings.REPOSITORIES.EDITOR.APT;

export default function AptDistributionConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <>
      <NxH2>{CAPTION}</NxH2>
      <NxFormGroup label={LABEL} sublabel={SUBLABEL} isRequired>
        <NxTextInput
          {...FormUtils.fieldProps('apt.distribution', parentState)}
          onChange={FormUtils.handleUpdate('apt.distribution', sendParent)}
          className="nx-text-input--long"
        />
      </NxFormGroup>
    </>
  );
}
