
import React from 'react';
import {AsTextLink} from '@globalart/react-shared-components';
import { faUsers } from '@fortawesome/free-solid-svg-icons';

export default {
  USERS: {
    MENU: {
      text: 'Users',
      description: 'Manage users',
      icon: faUsers
    },
    MODAL: {
      TEXT: 'You must confirm your current password before you are able to change or update the password',
      CHANGE_PASSWORD: 'Change Password',
      NEXT: 'Next',
      CONFIRM_PASSWORD: 'Confirm password',
      NEW_PASSWORD: 'New password',
      ADMIN_PASSWORD: 'Admin password',
      CONFIRMING_ADMIN_PASSWORD: 'Confirming Admin Password',
      ERROR: {
        MISSING_PASSWORD: 'Password was not supplied in the body of the request',
        PERMISSION: 'The user does not have permission to perform the operation.',
        NOT_FOUND: 'User not found in the system.'
      }
    },
    LIST: {
      CREATE_BUTTON: 'Create Local User',
      FILTER_PLACEHOLDER: 'Filter by user ID',
      EMPTY_LIST: 'There are no users available',
      COLUMNS: {
        USER_ID: 'User ID',
        REALM: 'Realm',
        FIRST_NAME: 'First Name',
        LAST_NAME: 'Last Name',
        EMAIL: 'Email',
        STATUS: 'Status',
      },
      HELP: {
        TITLE: 'What is a User?',
        TEXT: <>
          A user is an individual account attached to a single ID and email address.
          See our{' '}
          <AsTextLink external href="http://links.globalart.dev/products/astrum/docs/users">
            documentation
          </AsTextLink>
          {' '}for more information.
        </>,
      },
    },
    FORM: {
      CREATE_TITLE: 'Create User',
      EDIT_TILE: (name) => `Edit ${name}`,
      EDIT_DESCRIPTION: (source) => `${source} User`,
      LOCAL_USER_SOURCE: 'Local',
      DEFAULT_USER_WARNING: 'This is a default user and cannot be modified.',
      SECTIONS: {
        SETUP: 'User Setup',
        ROLES: 'Roles',
      },
      ID: {
        LABEL: 'ID',
        SUB_LABEL: 'This will be used as the username',
      },
      FIRST_NAME: {
        LABEL: 'First Name',
      },
      LAST_NAME: {
        LABEL: 'Last Name',
      },
      EMAIL: {
        LABEL: 'Email',
        SUB_LABEL: 'Used for notifications',
      },
      PASSWORD: {
        LABEL: 'Password',
      },
      CONFIRM_PASSWORD: {
        LABEL: 'Confirm password',
      },
      STATUS: {
        LABEL: 'Status',
        OPTIONS: {
          ACTIVE: 'Active',
          DISABLED: 'Disabled',
        }
      },
      ROLES: {
        AVAILABLE: 'Available',
        GRANTED: 'Granted',
        EMPTY_LIST: 'There are no assigned roles.',
      },
      EXTERNAL_ROLES: {
        LABEL: 'External Roles',
        EMPTY_LIST: 'No roles from the external system were mapped for this user.',
      },
    },
    MESSAGES: {
      CONFIRM_DELETE: {
        TITLE: 'Delete User',
        MESSAGE: (name) => `Are you sure you want to delete the user named "${name}?"`,
        YES: 'Delete',
        NO: 'Cancel'
      },
      DELETE_SUCCESS: (name) => `User deleted: ${name}`,
    },
    TOKEN: {
      LABEL: 'User Token Options',
      USER_TOKEN: 'User Token',
      ACTIVE_FEATURE: 'Resetting a user’s token will invalidate their current token and force a new token to be created the next time it is accessed by the user',
      REQUIRE_ENABLE_FEATURE: <>
        User tokens allow users to authenticate securely without typical user credentials such as those used by LDAP or Crowd. To use this feature, visit the <AsTextLink href="#admin/system/capabilities">
        capabilities page </AsTextLink> and create a “User Token” capability.
      </>,
      RESET_USER_TOKEN: 'Reset user token',
      TEXT: 'Resetting user tokens requires validation of your credentials.',
      AUTHENTICATE: 'Authenticate',
      SAVE_SUCCESS: (user) => `User token of ${user} has been reset`
    }
  }
};