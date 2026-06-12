
import { faUser } from '@fortawesome/free-solid-svg-icons';

export default {
  USER_ACCOUNT: {
    MENU: {
      text: 'My Account',
      description: 'Manage your account',
      icon: faUser
    },
    ACTIONS: {
      changePassword: 'Change password',
      discardChangePassword: 'Discard password change',
    },
    MESSAGES: {
      LOAD_ERROR: 'An error occurred while loading User Account, see console for more details',
      UPDATE_SUCCESS: 'User account settings updated',
      UPDATE_ERROR: 'An error occurred while updating user account settings',
      PASSWORD_MUST_DIFFER_ERROR: 'New password must be different',
      PASSWORD_CHANGE_SUCCESS: 'Password changed',
      PASSWORD_CHANGE_ERROR: 'Change password failed',
    },
    ID_FIELD_LABEL: 'Username',
    FIRST_FIELD_LABEL: 'First Name',
    LAST_FIELD_LABEL: 'Last Name',
    EMAIL_FIELD_LABEL: 'Email',
    PASSWORD_CURRENT_FIELD_LABEL: 'Current Password',
    PASSWORD_NEW_FIELD_LABEL: 'New Password',
    PASSWORD_NEW_CONFIRM_FIELD_LABEL: 'New Password (Confirm)',
  }
};