
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsCheckbox,
  AsFieldset,
  AsTextLink,
  AsWarningAlert
} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {
  NPM: {REMOVE_QUARANTINED},
  LEARN_MORE
} = UIStrings.REPOSITORIES.EDITOR;

const getSubLabel = (txt) => (
  <>
    {txt}{' '}
    <AsTextLink href="http://links.globalart.dev/products/astrum/docs/npm-with-firewall" external>
      {LEARN_MORE}
    </AsTextLink>
  </>
);

export default function NpmConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  return (
    <>
      <AsFieldset
        label={REMOVE_QUARANTINED.LABEL}
        sublabel={getSubLabel(REMOVE_QUARANTINED.SUBLABEL)}
        className="as-form-group-remove-quarantined"
      >
        <AsCheckbox
          {...FormUtils.checkboxProps('npm.removeQuarantined', currentParent)}
          onChange={FormUtils.handleUpdate('npm.removeQuarantined', sendParent)}
        >
          {REMOVE_QUARANTINED.DESCR}
        </AsCheckbox>
        <AsWarningAlert>{REMOVE_QUARANTINED.WARNING}</AsWarningAlert>
      </AsFieldset>
    </>
  );
}
