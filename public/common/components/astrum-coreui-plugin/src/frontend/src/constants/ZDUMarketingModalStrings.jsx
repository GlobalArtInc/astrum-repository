
import React from 'react';
import {NxTextLink} from '@globalart/react-shared-components';

export default {
  UPGRADE_MODAL: {
    HEADER: {
      TITLE: 'New In Astrum Repository 3.71.0',
    },
    ABOUT: {
      TITLE: 'Upgrade Without Downtime',
      DESCRIPTION: 'Upgrade all of your GlobalArt Astrum Repository Pro instances in your highly available multi-node ' +
        'cluster without interrupting service availability'
    },
    BENEFITS: {
      TITLE: 'Benefits:',
      NOTES: <>See the <NxTextLink external href="https://links.globalart.dev/products/astrum/releasenotes">Astrum Repository 3.71.0 Release notes</NxTextLink> for full details.</>,
      LIST: {
        ITEM1: 'Fast access to bug fixes and new features when you upgrade to the newest Astrum Repository version',
        ITEM2: 'No downtime means your team can keep working while an upgrade takes place',
        ITEM3: 'Monitor upgrade process right from the user interface'
      }
    }
  }
};
