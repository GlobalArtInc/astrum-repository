
import { faKey } from '@fortawesome/free-solid-svg-icons';

export default {
  USER_TOKEN: {
    MENU: {
      text: 'User Token',
      description: 'Access GlobalArt Astrum Repository without the use of passwords',
      icon: faKey
    },
    CAPTION: 'Token Information',
    LABELS: {
      NOTE: 'Astrum Repository creates a new user token when a user first accesses that token. ' +
          'Resetting your user token invalidates the current token. Astrum Repository will create a new user token when you next access that token.',
      USER_TOKEN_NOTE: 'User tokens are a combination of a name and password codes.',
      KEEP_SECRET_NOTE: 'Keep these codes secret.',
      USER_TOKEN_NAME_CODE: 'Your user token name code is',
      USER_TOKEN_PASS_CODE: 'Your user token pass code is',
      MAVEN_USAGE: 'Use the following in your Maven settings.xml',
      BASE64_USER_TOKEN: 'Use the following for a base64 representation of "user:password"',
      AUTO_HIDE: 'This window will automatically close after one minute.',
    },
    MESSAGES: {
      ACCESS_AUTHENTICATION: 'Accessing user tokens requires validation of your credentials',
      RESET_AUTHENTICATION: 'Resetting user tokens requires validation of your credentials',
      ACCESS_ERROR: 'You must authenticate successfully to access your token',
      RESET_SUCCESS: 'Your user token has been reset',
      RESET_ERROR: 'You must authenticate successfully to reset your token',
      GENERATE_AUTHENTICATION: 'Generating user tokens requires validation of your credentials',
      GENERATE_ERROR: 'You must authenticate successfully to generate your token',
      EXPIRED: 'User Token expired'
    },
    BUTTONS: {
      GENERATE: 'Generate User Token',
      ACCESS: 'Access User Token',
      RESET: 'Reset User Token',
      CLOSE: 'Close'
    },
    USER_TOKEN_STATUS: {
      TEXT: 'User Token Status',
      DESCRIPTION: 'Time remaining until user token expires',
      TIMESTAMP_TEXT: (isExpired) => isExpired ? 'Expired' : 'Expires: ',
      EXPIRED_WARNING: 'Your user token has expired. Select generate user token to create a new one.'
    }
  }
};
