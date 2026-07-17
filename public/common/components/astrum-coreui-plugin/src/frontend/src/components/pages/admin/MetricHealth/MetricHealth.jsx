
import React from 'react';
import {Detail, Master, MasterDetail} from '@globalart/astrum-ui-plugin';

import MetricHealthList from './MetricHealthList';
import MetricHealthDetails from './MetricHealthDetails';
import {isClustered} from './MetricHealthHelper';

export default function MetricHealth() {
  return (
    <MasterDetail path="admin/support/status">
      <Master>
        {isClustered() ? <MetricHealthList /> : <MetricHealthDetails />}
      </Master>
      <Detail>
        <MetricHealthDetails />
      </Detail>
    </MasterDetail>
  );
}
