import React from 'react';
import {NxPageMain} from '@globalart/react-shared-components';

import OutreachActions from './OutreachActions';

import './Welcome.scss';

export default function Welcome() {
  return (
      <NxPageMain>
        <div className="as-welcome">
          <div className="as-welcome__outreach nx-viewport-sized__scrollable">
            <OutreachActions/>
          </div>
        </div>
      </NxPageMain>
  );
}
