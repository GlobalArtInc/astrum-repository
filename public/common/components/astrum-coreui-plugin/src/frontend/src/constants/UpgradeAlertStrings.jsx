
import React from 'react';
import {AsTextLink} from '@globalart/astrum-react-shared-components';

export default {
  UPGRADE_ALERT: {
    PENDING: {
      LABEL: 'Upgrade Pending.',
      FINALIZE_BUTTON: 'Finalize Upgrade',
      STATUS_LABEL: 'Complete Astrum Repository version upgrade.',
      TEXT: 'New features and functionality will not be available until the upgrade process is complete.',
      OLDER_LABEL: 'Astrum Repository versioning is out of date.',
      YEAR_OLD_TEXT: <>You are currently running a GlobalArt Astrum Repository version that is in extended maintenance.
        See our <AsTextLink external href="https://links.globalart.dev/products/sunsetting/docs/information">
        Sunsetting information</AsTextLink> for details about our product
        development lifecycle. Also see our <AsTextLink external href="https://links.globalart.dev/products/astrum/releasenotes">
        release notes</AsTextLink> to learn more about the newest Astrum Repository version available for your instance.</>,
      TEXT_18_MONTHS: <>You are currently running a sunsetted GlobalArt Astrum Repository version.
        See our <AsTextLink external href="https://links.globalart.dev/products/sunsetting/docs/information">
        Sunsetting information</AsTextLink> for details about our product development
        lifecycle. Also see our <AsTextLink external href="https://links.globalart.dev/products/astrum/releasenotes">
        release notes</AsTextLink> to learn more about the newest Astrum Repository version available for your instance.</>
    },

    PROGRESS: {
      LABEL: 'Upgrade in progress'
    },

    WARN: {
      LABEL: 'Mixed Mode detected.',
      TEXT: 'Astrum Repository detected that 1 or more node(s) are running different Astrum Repository versions. ' +
        'This is not supported except briefly during the upgrade process. ' +
        'Remaining in mixed mode outside of an active upgrade process can result in unexpected behavior and Astrum Repository becoming unstable. ' + 
        'Ensure all nodes are on the same version as soon as possible.'
    },

    ERROR: {
      LABEL: 'Upgrade Failed.',
      TEXT: 'error occurred while trying update the nodes in your cluster.',
      TEXT_MISMATCH: 'or more node(s) in your cluster is not running the same version of Astrum Repository.',
      CONTACT_SUPPORT: {
        HREF: 'https://links.globalart.dev/products/astrum/pro/support-request',
        TEXT: 'Contact support'
      },
      ASSISTANCE: 'for assistance.'
    },

    COMPLETE: {
      LABEL: 'Upgrade Complete.',
      TEXT: 'Any new functionality included in your upgrade is now available for all nodes in your cluster.',
      DISMISS: 'Dismiss'
    },
  }
};
