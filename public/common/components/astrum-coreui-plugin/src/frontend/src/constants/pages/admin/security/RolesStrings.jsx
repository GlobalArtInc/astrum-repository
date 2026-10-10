
import React from 'react';
import {AsTextLink} from '@globalart/astrum-react-shared-components';
import { faUserTag } from '@fortawesome/free-solid-svg-icons';

export default {
  ROLES: {
    MENU: {
      text: 'Roles',
      description: 'Manage roles',
      icon: faUserTag
    },
    LIST: {
      CREATE_BUTTON: 'Create Role',
      EMPTY_LIST: 'There are no roles available',
      COLUMNS: {
        ID: 'Id',
        NAME: 'Name',
        DESCRIPTION: 'Description',
      },
      HELP: {
        TITLE: 'What is a role?',
        TEXT: <>
          Roles bring together multiple privileges so that, when you assign a user to the role,
          that user will automatically have all of those privileges. Roles can comprise both
          other roles and individual privileges. See our{' '}
          <AsTextLink external href="http://links.globalart.dev/products/astrum/docs/roles">
            help documentation
          </AsTextLink>
          {' '}for more information.
        </>,
      },
      ALERT: {
        DEFAULT_ROLE: (roleId, roleName) => <>
          This instance is using{' '}
          <AsTextLink href={`#admin/security/roles:${roleId}`}>
            {roleName}
          </AsTextLink>
          {' '}as the default role granted to all authenticated users.
        </>,
        CAPABILITY: (capabilityId) => <>
          To manage this configuration, see the{' '}
          <AsTextLink href={`#admin/system/capabilities:${capabilityId}`}>
            Default Role capability
          </AsTextLink>
          {' '}page.
        </>
      },
    },
    FORM: {
      CREATE_TITLE: 'Create Role',
      EDIT_TILE: (name) => `Edit ${name}`,
      EDIT_DESCRIPTION: 'Edit role configuration',
      DEFAULT_ROLE_WARNING: 'This is a default role and cannot be modified.',
      SECTIONS: {
        TYPE: 'Role Type',
        SETUP: 'Role Setup',
        PRIVILEGES: 'Privileges',
        ROLES: 'Roles',
      },
      TYPE: {
        LABEL: 'Type',
        OPTIONS: {
          ASTRUM: 'Astrum role',
          EXTERNAL: 'External Role Mapping',
        }
      },
      EXTERNAL_TYPE: {
        LABEL: 'External Role Type', 
        LDAP: {
          MORE_CHARACTERS: (value) => `Enter ${value} or more characters`,
          NO_RESULTS: 'If no results are found, you can still save this mapping and create an LDAP ' +
            'role with an identical name later. Mapping will not apply until an identically named LDAP role exists.'
        }
      },
      ID: {
        LABEL: 'Role ID',
      },
      MAPPED_ROLE: {
        LABEL: 'Mapped Role',
      },
      NAME: {
        LABEL: 'Role Name',
      },
      DESCRIPTION: {
        LABEL: 'Role Description',
      },
      PRIVILEGES: {
        LABEL: 'Privileges',
        AVAILABLE: 'Available',
        SELECTED: 'Given',
        EMPTY_LIST: 'This role does not have any privileges.',
      },
      ROLES: {
        LABEL: 'Roles',
        AVAILABLE: 'Available',
        SELECTED: 'Contained',
        EMPTY_LIST: 'This role does not contain any other roles.',
      },
    },
    MESSAGES: {
      CONFIRM_DELETE: {
        TITLE: 'Delete Role',
        MESSAGE: (name) => `Are you sure you want to delete the role named "${name}?"`,
        YES: 'Delete',
        NO: 'Cancel'
      },
      DELETE_SUCCESS: (name) => `Role deleted: ${name}`,
    },
    SELECTION_MODAL: {
      WILDCARD_TEXT: <i>Use * as a wildcard</i>
    }
  }
};
