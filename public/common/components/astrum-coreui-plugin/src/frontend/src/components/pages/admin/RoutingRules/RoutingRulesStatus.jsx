
import React from 'react';
import {faCheckCircle, faTimesCircle} from '@fortawesome/free-solid-svg-icons';

import {NxFontAwesomeIcon} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {ROUTING_RULES} = UIStrings;

export default function RoutingRulesStatus({allowed}) {
  if (allowed) {
    return <span className="allowed">
      <NxFontAwesomeIcon icon={faCheckCircle}/>
      <span>{ROUTING_RULES.ALLOWED}</span>
    </span>;
  }
  else {
    return <span className="blocked">
      <NxFontAwesomeIcon icon={faTimesCircle}/>
      <span>{ROUTING_RULES.BLOCKED}</span>
    </span>;
  }
}
