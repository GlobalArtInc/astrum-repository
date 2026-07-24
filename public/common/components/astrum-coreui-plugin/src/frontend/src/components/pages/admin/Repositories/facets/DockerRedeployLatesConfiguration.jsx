
import React, {useEffect} from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxCheckbox, NxFieldset, NxTooltip} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {LABEL, DESCRIPTION, TOOLTIP} = UIStrings.REPOSITORIES.EDITOR.REDEPLOY_LATEST;

const ALLOW_ONCE = 'ALLOW_ONCE';

const PROP_PATH = 'storage.latestPolicy';

export default function DockerRedeployLatesConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  const {writePolicy} = parentState.context.data.storage;

  useEffect(() => {
    if (writePolicy === ALLOW_ONCE) {
      sendParent({type: 'ADD_DATA_PROPERTY', path: PROP_PATH, value: false});
    } else {
      sendParent({type: 'DELETE_DATA_PROPERTY', path: PROP_PATH});
    }
  }, [writePolicy]);

  const isDisabled = writePolicy !== ALLOW_ONCE;

  return (
    <NxFieldset label={LABEL}>
      <NxTooltip title={isDisabled ? TOOLTIP : null}>
        <NxCheckbox
          {...FormUtils.checkboxProps(PROP_PATH, parentState)}
          onChange={FormUtils.handleUpdate(PROP_PATH, sendParent)}
          disabled={isDisabled}
          overflowTooltip={!isDisabled}
        >
          {DESCRIPTION}
        </NxCheckbox>
      </NxTooltip>
    </NxFieldset>
  );
}
