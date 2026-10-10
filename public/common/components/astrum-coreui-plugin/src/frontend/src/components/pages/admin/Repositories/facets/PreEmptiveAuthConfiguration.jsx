
import React, {useEffect} from 'react';

import {FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import {AsCheckbox, AsFieldset, AsTooltip} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

import './PreEmptiveAuthConfiguration.scss';

const {LABEL, DESCR, TOOLTIP} = UIStrings.REPOSITORIES.EDITOR.PRE_EMPTIVE_AUTH;

const PROP_PATH = 'httpClient.authentication.preemptive';

export default function PreEmptiveAuthConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  const {
    proxy: {remoteUrl},
    httpClient: {authentication}
  } = parentState.context.data;

  const hasSecureRemoteUrl = ValidationUtils.isSecureUrl(remoteUrl);
  const isDisabled = !hasSecureRemoteUrl;
  const isVisible = !!authentication;

  useEffect(() => {
    if (isVisible && !isDisabled) {
      sendParent({type: 'ADD_DATA_PROPERTY', path: PROP_PATH, value: false});
    } else {
      sendParent({type: 'DELETE_DATA_PROPERTY', path: PROP_PATH});
    }
  }, [isVisible, isDisabled]);

  return (
    <>
      {isVisible && (
        <AsFieldset label={LABEL}>
          <AsTooltip title={isDisabled ? TOOLTIP : null}>
            <AsCheckbox
              {...FormUtils.checkboxProps(PROP_PATH, parentState)}
              onChange={FormUtils.handleUpdate(PROP_PATH, sendParent)}
              disabled={isDisabled}
              overflowTooltip={!isDisabled}
            >
              {DESCR}
            </AsCheckbox>
          </AsTooltip>
        </AsFieldset>
      )}
    </>
  );
}
