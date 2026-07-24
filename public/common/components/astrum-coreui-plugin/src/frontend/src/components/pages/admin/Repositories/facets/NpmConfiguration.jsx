
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  NxCheckbox,
  NxFieldset,
  NxTextLink,
  NxWarningAlert
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {
  NPM: {REMOVE_QUARANTINED},
  LEARN_MORE
} = UIStrings.REPOSITORIES.EDITOR;

const getSubLabel = (txt) => (
  <>
    {txt}{' '}
    <NxTextLink href="http://links.globalart.dev/products/astrum/docs/npm-with-firewall" external>
      {LEARN_MORE}
    </NxTextLink>
  </>
);

export default function NpmConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  return (
    <>
      <NxFieldset
        label={REMOVE_QUARANTINED.LABEL}
        sublabel={getSubLabel(REMOVE_QUARANTINED.SUBLABEL)}
        className="as-form-group-remove-quarantined"
      >
        <NxCheckbox
          {...FormUtils.checkboxProps('npm.removeQuarantined', currentParent)}
          onChange={FormUtils.handleUpdate('npm.removeQuarantined', sendParent)}
        >
          {REMOVE_QUARANTINED.DESCR}
        </NxCheckbox>
        <NxWarningAlert>{REMOVE_QUARANTINED.WARNING}</NxWarningAlert>
      </NxFieldset>
    </>
  );
}
