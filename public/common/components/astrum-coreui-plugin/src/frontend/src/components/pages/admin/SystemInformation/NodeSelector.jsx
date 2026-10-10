
import React from 'react';
import {useActor} from '@xstate/react';

import {AsFormGroup, AsFormSelect} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {ACTIONS, LOAD_ERROR} = UIStrings.SYSTEM_INFORMATION;

export default function NodeSelector({service}) {
  const [state, send] = useActor(service);

  const isLoading = state.matches('loading');
  const loadError = state.matches('loadError') ? LOAD_ERROR : null;

  const {selectedNodeId, nodeIds} = state.context;

  const handleSelectNode = (value) => {
    send({type: 'SELECT_NODE', value});
  };

  return (
    <div className="as-sys-info-node-selector">
      <AsFormGroup label={ACTIONS.nodeSelector}>
        <AsFormSelect
          onChange={handleSelectNode}
          value={selectedNodeId}
          disabled={isLoading || loadError}
        >
          {nodeIds.map((nodeId) => (
            <option value={nodeId} key={nodeId}>
              {nodeId}
            </option>
          ))}
        </AsFormSelect>
      </AsFormGroup>
    </div>
  );
}
