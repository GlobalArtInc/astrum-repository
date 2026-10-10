
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsTextInput} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {KEY, PASSPHRASE} = UIStrings.REPOSITORIES.EDITOR.ALPINE.SIGNING;

export default function AlpineSigningConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <>
      <AsFormGroup label={KEY.LABEL} sublabel={KEY.SUBLABEL} isRequired>
        <AsTextInput
          type="textarea"
          {...FormUtils.fieldProps('alpineSigning.keypair', parentState)}
          onChange={FormUtils.handleUpdate('alpineSigning.keypair', sendParent)}
          className="as-text-input--long"
          placeholder={KEY.PLACEHOLDER}
        />
      </AsFormGroup>

      <AsFormGroup label={PASSPHRASE.LABEL} sublabel={PASSPHRASE.SUBLABEL}>
        <AsTextInput
          type="password"
          {...FormUtils.fieldProps('alpineSigning.passphrase', parentState)}
          onChange={FormUtils.handleUpdate('alpineSigning.passphrase', sendParent)}
        />
      </AsFormGroup>
    </>
  );
}
