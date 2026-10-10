
import React from 'react';
import {AsTextLink} from '@globalart/astrum-react-shared-components';
import { faIdBadge } from '@fortawesome/free-solid-svg-icons';

export default {
  PRIVILEGES: {
    MENU: {
      text: 'Privileges',
      description: 'Manage Privileges',
      icon: faIdBadge
    },
    LIST: {
      CREATE_BUTTON: 'Create Privilege',
      EMPTY_LIST: 'There are no privileges available',
      COLUMNS: {
        NAME: 'Name',
        DESCRIPTION: 'Description',
        TYPE: 'Type',
        PERMISSION: 'Permission',
      },
      HELP: {
        TITLE: 'What is a Privilege?',
        TEXT: <>
          Privileges are assigned to roles and control the actions that each role can perform.
          See our{' '}
          <AsTextLink external href="http://links.globalart.dev/products/astrum/docs/privileges">
            documentation
          </AsTextLink>
          {' '}for more information.
        </>,
      },
    },
    FORM: {
      CREATE_TITLE: 'Create Privilege',
      EDIT_TILE: (name) => `Edit ${name}`,
      EDIT_DESCRIPTION: 'Astrum Privilege',
      DEFAULT_PRIVILEGE_WARNING: 'This is a default privilege and cannot be modified.',
      SECTIONS: {
        SETUP: 'Privilege Setup',
      },
      TYPE: {
        LABEL: 'Type',
      },
      NAME: {
        LABEL: 'Name',
      },
      DESCRIPTION: {
        LABEL: 'Description',
      },
      FORMAT: {
        LABEL: 'Format',
        SUB_LABEL: 'The format(s) for the repository',
      },
      PRIVILEGE_STRING: {
        LABEL: 'Privilege String',
        SUB_LABEL: <>
          The internal segment matching algorithm uses Apache Shiro wildcard <br />permissions; see{' '}
          <AsTextLink external href="http://links.globalart.dev/products/astrum/docs/privileges">
            our documentation
          </AsTextLink>
          {' '}for more details
        </>,
      },
    },
    MESSAGES: {
      CONFIRM_DELETE: {
        TITLE: 'Delete Privilege',
        MESSAGE: (name) => `Are you sure you want to delete the privilege named "${name}?"`,
        YES: 'Delete',
        NO: 'Cancel'
      },
      DELETE_SUCCESS: (name) => `Privilege deleted: ${name}`,
      NO_ACTION_ERROR: 'At least one action must be selected'
    },
  }
};
