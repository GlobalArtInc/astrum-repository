
import { faUser } from '@fortawesome/free-solid-svg-icons';

export default {
  ANONYMOUS_SETTINGS: {
    MENU: {
      text: 'Anonymous Access',
      description: 'Configure anonymous access to server contents',
      icon: faUser
    },
    ENABLED_CHECKBOX_LABEL: 'Access',
    ENABLED_CHECKBOX_DESCRIPTION: 'Allow anonymous users to access the server',
    USERNAME_TEXTFIELD_LABEL: 'Username',
    REALM_SELECT_LABEL: 'Realm',
    MESSAGES: {
      LOAD_ERROR: 'An error occurred while loading Anonymous settings, see console for more details',
      SAVE_SUCCESS: 'Anonymous security settings updated',
      SAVE_ERROR: 'An error occurred while updating Anonymous settings, see console for more details'
    }
  }
};