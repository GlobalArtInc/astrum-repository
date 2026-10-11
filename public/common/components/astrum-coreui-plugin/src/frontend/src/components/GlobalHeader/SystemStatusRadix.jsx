import React from 'react';
import {SystemStatusBell} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../constants/UIStrings';

import './SystemStatusRadix.scss';

const {HEADER} = UIStrings.SHELL;

/**
 * The system status bell of the header, drawn as the same bordered square button as its neighbours.
 */
export default function SystemStatusRadix() {
  return (
    <SystemStatusBell
      className="as-system-status-radix"
      buttonClassName="as-global-header-radix__icon-button"
      variant="outline"
      size="3"
      label={HEADER.SYSTEM_STATUS}
      unhealthyLabel={HEADER.SYSTEM_STATUS_UNHEALTHY}
    />
  );
}
