
import React from 'react';

import {Detail, Master, MasterDetail} from '@globalart/astrum-ui-plugin';

import LogsList from './LogsList';
import LogViewer from './LogViewer';

import './Logs.scss';

export default function Logs() {
  return <MasterDetail path="admin/support/logs">
    <Master>
      <LogsList/>
    </Master>
    <Detail>
      <LogViewer/>
    </Detail>
  </MasterDetail>;
}
