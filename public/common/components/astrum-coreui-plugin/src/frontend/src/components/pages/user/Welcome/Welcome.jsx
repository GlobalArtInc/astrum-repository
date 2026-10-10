import React from 'react';
import {AsPageMain} from '@globalart/astrum-react-shared-components';

import OutreachActions from './OutreachActions';

import './Welcome.scss';

export default function Welcome() {
  return (
      <AsPageMain>
        <div className="as-welcome">
          <div className="as-welcome__outreach as-viewport-sized__scrollable">
            <OutreachActions/>
          </div>
        </div>
      </AsPageMain>
  );
}
