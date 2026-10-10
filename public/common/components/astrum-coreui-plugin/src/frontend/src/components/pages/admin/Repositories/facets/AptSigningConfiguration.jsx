
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsTextInput} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {KEY, PASSPHRASE} = UIStrings.REPOSITORIES.EDITOR.APT.SIGNING;

export default function AptSigningConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <>
      <AsFormGroup label={KEY.LABEL} sublabel={KEY.SUBLABEL} isRequired>
        <AsTextInput
          type="textarea"
          {...FormUtils.fieldProps('aptSigning.keypair', parentState)}
          onChange={FormUtils.handleUpdate('aptSigning.keypair', sendParent)}
          className="as-text-input--long"
          placeholder={KEY.PLACEHOLDER}
        />
      </AsFormGroup>

      <AsFormGroup label={PASSPHRASE.LABEL} sublabel={PASSPHRASE.SUBLABEL}>
        <AsTextInput
          type="password"
          {...FormUtils.fieldProps('aptSigning.passphrase', parentState)}
          onChange={FormUtils.handleUpdate('aptSigning.passphrase', sendParent)}
        />
      </AsFormGroup>
    </>
  );
}
