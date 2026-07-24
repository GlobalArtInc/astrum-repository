
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxTextInput} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {KEY, PASSPHRASE} = UIStrings.REPOSITORIES.EDITOR.ALPINE.SIGNING;

export default function AlpineSigningConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <>
      <NxFormGroup label={KEY.LABEL} sublabel={KEY.SUBLABEL} isRequired>
        <NxTextInput
          type="textarea"
          {...FormUtils.fieldProps('alpineSigning.keypair', parentState)}
          onChange={FormUtils.handleUpdate('alpineSigning.keypair', sendParent)}
          className="nx-text-input--long"
          placeholder={KEY.PLACEHOLDER}
        />
      </NxFormGroup>

      <NxFormGroup label={PASSPHRASE.LABEL} sublabel={PASSPHRASE.SUBLABEL}>
        <NxTextInput
          type="password"
          {...FormUtils.fieldProps('alpineSigning.passphrase', parentState)}
          onChange={FormUtils.handleUpdate('alpineSigning.passphrase', sendParent)}
        />
      </NxFormGroup>
    </>
  );
}
