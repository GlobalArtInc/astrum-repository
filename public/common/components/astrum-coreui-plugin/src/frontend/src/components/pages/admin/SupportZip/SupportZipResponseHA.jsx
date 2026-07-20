
import React from 'react';
import {NxStatefulTabs, NxTabList, NxTab, NxTabPanel} from '@globalart/react-shared-components';

import NodeSupportZipResponse from "./NodeSupportZipResponse";

export default function SupportZipResponseHA({response, download}) {
  return <NxStatefulTabs defaultActiveTab={0}>
    <NxTabList>
     {response.map((nodeZip) =>
       <NxTab id={nodeZip.nodeId} key={nodeZip.nodeId}>
         {nodeZip.nodeAlias}
       </NxTab>)}
    </NxTabList>
    {response.map((nodeZip) =>
        <NxTabPanel id={nodeZip.nodeId} key={nodeZip.nodeId}>
          <NodeSupportZipResponse response={nodeZip} download={download} />
        </NxTabPanel>
    )}
  </NxStatefulTabs>;
}
