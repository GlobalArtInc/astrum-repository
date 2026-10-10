
import React, {useEffect} from 'react';
import {assign} from 'xstate';

import {ExtAPIUtils, FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsLoadWrapper, AsStatefulTransferList} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function GenericCleanupConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;
  const [cleanupState, cleanupSend] = ExtAPIUtils.useExtMachine('cleanup_CleanupPolicy', 'readByFormat', {
    actions: {
      setData: assign({
        data: (_, {data}) => data?.map(policy =>
            ({id: policy.name, displayName: policy.name}))
      })
    }
  });

  const {format, cleanup} = currentParent.context.data;
  const {data: availablePolicies = [], error} = cleanupState.context;
  const isLoading = cleanupState.matches('loading');

  const loadCleanupPolicies = () => cleanupSend({
    type: 'LOAD',
    options: {
      filterField: 'format',
      filterValue: format,
    },
  });

  useEffect(() => {
    loadCleanupPolicies();
  }, [format]);

  return (
    <>
      <h2 className="as-h2">{EDITOR.CLEANUP_CAPTION}</h2>
      <AsLoadWrapper loading={isLoading} error={error} retryHandler={loadCleanupPolicies}>
        <AsFormGroup
          label={EDITOR.CLEANUP_POLICIES_LABEL}
          sublabel={EDITOR.CLEANUP_POLICIES_SUBLABEL}
        >
          <AsStatefulTransferList
            allItems={availablePolicies}
            selectedItems={new Set(cleanup?.policyNames)}
            onChange={FormUtils.handleUpdate('cleanup.policyNames', sendParent)}
          />
        </AsFormGroup>
      </AsLoadWrapper>
    </>
  );
}
