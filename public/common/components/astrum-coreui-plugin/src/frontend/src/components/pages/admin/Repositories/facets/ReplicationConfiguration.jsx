
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsCheckbox, AsFieldset} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function ReplicationConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  return (
    <AsFieldset
      label={EDITOR.REPLICATION_LABEL}
      sublabel={EDITOR.REPLICATION_SUBLABEL}
      className="as-form-group-replication"
    >
      <AsCheckbox
        {...FormUtils.checkboxProps('replication.enabled', currentParent)}
        onChange={FormUtils.handleUpdate('replication.enabled', sendParent)}
      >
        {EDITOR.ENABLED_CHECKBOX_DESCR}
      </AsCheckbox>
    </AsFieldset>
  );
}
