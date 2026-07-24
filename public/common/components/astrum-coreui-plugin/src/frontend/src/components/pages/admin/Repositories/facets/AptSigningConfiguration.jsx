
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxTextInput} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {KEY, PASSPHRASE} = UIStrings.REPOSITORIES.EDITOR.APT.SIGNING;

export default function AptSigningConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <>
      <NxFormGroup label={KEY.LABEL} sublabel={KEY.SUBLABEL} isRequired>
        <NxTextInput
          type="textarea"
          {...FormUtils.fieldProps('aptSigning.keypair', parentState)}
          onChange={FormUtils.handleUpdate('aptSigning.keypair', sendParent)}
          className="nx-text-input--long"
          placeholder={KEY.PLACEHOLDER}
        />
      </NxFormGroup>

      <NxFormGroup label={PASSPHRASE.LABEL} sublabel={PASSPHRASE.SUBLABEL}>
        <NxTextInput
          type="password"
          {...FormUtils.fieldProps('aptSigning.passphrase', parentState)}
          onChange={FormUtils.handleUpdate('aptSigning.passphrase', sendParent)}
        />
      </NxFormGroup>
    </>
  );
}
