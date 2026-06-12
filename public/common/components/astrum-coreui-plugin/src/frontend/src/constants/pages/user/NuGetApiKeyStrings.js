
import { faKey } from '@fortawesome/free-solid-svg-icons';

export default {
  NUGET_API_KEY: {
    MENU: {
      text: 'NuGet API Key',
      description: 'Configure credentials for NuGet repositories',
      icon: faKey
    },
    INSTRUCTIONS: 'A new API Key will be created the first time it is accessed. Resetting your API Key will invalidate the current key.',
    AUTH_INSTRUCTIONS: 'Accessing NuGet API Key requires validation of your credentials.',
    AUTH_ERROR: 'Authentication Failure',
    CLOSE: 'Close',
    ACCESS: {
      BUTTON: 'Access API Key',
      HELP_TEXT: 'Accessing NuGet API Key requires validation of your credentials.',
      ERROR: 'Failed to access API Key'
    },
    RESET: {
      BUTTON: 'Reset API Key',
      HELP_TEXT: 'Resetting NuGet API Key requires validation of your credentials.',
      ERROR: 'Failed to Reset API Key'
    },
    DETAILS: {
      MAIN: 'Your NuGet API Key enables pushing packages using nuget.exe.',
      WARNING: 'Keep this key secret!',
      API_KEY_TEXT: 'Your NuGet API Key is',
      REGISTER_TEXT: 'You can register this key for a given repository with the following command',
      REGISTER_COMMAND: 'nuget setapikey {0} -source {1}',
      AUTO_CLOSE: 'This window will automatically close after one minute.'
    },
    MESSAGES: {
      ACCESS_ERROR: 'You must authenticate successfully to access your NuGet API Key',
      RESET_SUCCESS: 'Your user NuGet API Key has been reset',
      RESET_ERROR: 'You must authenticate successfully to reset your NuGet API Key'
    }
  }
};
