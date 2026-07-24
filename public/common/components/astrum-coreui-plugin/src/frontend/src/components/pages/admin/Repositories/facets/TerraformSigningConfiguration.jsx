
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  NxFormGroup,
  NxTextInput
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {KEY, PASSPHRASE, CAPTION} = UIStrings.REPOSITORIES.EDITOR.TERRAFORM.SIGNING;

export default function TerraformSigningConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <>
      <h2 className="nx-h2">{CAPTION}</h2>
      <NxFormGroup label={KEY.LABEL} sublabel={KEY.SUBLABEL} isRequired>
        <NxTextInput
          type="textarea"
          {...FormUtils.fieldProps('terraformSigning.keypair', parentState)}
          onChange={FormUtils.handleUpdate('terraformSigning.keypair', sendParent)}
          className="nx-text-input--long"
          placeholder={KEY.PLACEHOLDER}
        />
      </NxFormGroup>

      <NxFormGroup label={PASSPHRASE.LABEL} sublabel={PASSPHRASE.SUBLABEL}>
        <NxTextInput
          type="password"
          {...FormUtils.fieldProps('terraformSigning.passphrase', parentState)}
          onChange={FormUtils.handleUpdate('terraformSigning.passphrase', sendParent)}
        />
      </NxFormGroup>
    </>
  );
}
