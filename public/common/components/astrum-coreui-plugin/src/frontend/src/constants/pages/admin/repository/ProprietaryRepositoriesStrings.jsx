
import React from 'react';
import {AsTextLink} from '@globalart/react-shared-components';
import { faDoorClosed } from '@fortawesome/free-solid-svg-icons';

export default {
  PROPRIETARY_REPOSITORIES: {
    MENU: {
      text: 'Proprietary Repositories',
      description: 'Manage the set of hosted repositories that contain proprietary components',
      icon: faDoorClosed
    },
    CONFIGURATION: {
      LABEL: 'Proprietary Repositories Configuration',
      AVAILABLE_TITLE: 'Generic Hosted Repositories',
      SELECTED_TITLE: 'Proprietary Hosted Repositories',
      EMPTY_LIST: 'There are no configured proprietary hosted repositories for which you have view permissions.',
    },
    HELP_TEXT: <>
      To help prevent dependency confusion attacks, identify your hosted repositories that contain
      proprietary components. Refer to the{' '}
      <AsTextLink external href="http://links.globalart.dev/products/astrum/docs/proprietary-repositories">
        documentation
      </AsTextLink>
      {' '}for details on setting up appropriate IQ policies to quarantine public components with the same names
      as your proprietary components.
    </>,
  }
};