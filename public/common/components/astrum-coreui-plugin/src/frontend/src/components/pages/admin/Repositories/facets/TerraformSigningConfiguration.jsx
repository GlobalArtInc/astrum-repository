
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsFormGroup,
  AsTextInput
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {KEY, PASSPHRASE, CAPTION} = UIStrings.REPOSITORIES.EDITOR.TERRAFORM.SIGNING;

export default function TerraformSigningConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  return (
    <>
      <h2 className="as-h2">{CAPTION}</h2>
      <AsFormGroup label={KEY.LABEL} sublabel={KEY.SUBLABEL} isRequired>
        <AsTextInput
          type="textarea"
          {...FormUtils.fieldProps('terraformSigning.keypair', parentState)}
          onChange={FormUtils.handleUpdate('terraformSigning.keypair', sendParent)}
          className="as-text-input--long"
          placeholder={KEY.PLACEHOLDER}
        />
      </AsFormGroup>

      <AsFormGroup label={PASSPHRASE.LABEL} sublabel={PASSPHRASE.SUBLABEL}>
        <AsTextInput
          type="password"
          {...FormUtils.fieldProps('terraformSigning.passphrase', parentState)}
          onChange={FormUtils.handleUpdate('terraformSigning.passphrase', sendParent)}
        />
      </AsFormGroup>
    </>
  );
}
