
import React from 'react';
import {AsStatefulTabs, AsTabList, AsTab, AsTabPanel} from '@globalart/react-shared-components';

import NodeSupportZipResponse from "./NodeSupportZipResponse";

export default function SupportZipResponseHA({response, download}) {
  return <AsStatefulTabs defaultActiveTab={0}>
    <AsTabList>
     {response.map((nodeZip) =>
       <AsTab id={nodeZip.nodeId} key={nodeZip.nodeId}>
         {nodeZip.nodeAlias}
       </AsTab>)}
    </AsTabList>
    {response.map((nodeZip) =>
        <AsTabPanel id={nodeZip.nodeId} key={nodeZip.nodeId}>
          <NodeSupportZipResponse response={nodeZip} download={download} />
        </AsTabPanel>
    )}
  </AsStatefulTabs>;
}
