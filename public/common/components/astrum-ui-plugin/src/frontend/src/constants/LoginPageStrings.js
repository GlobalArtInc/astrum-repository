

import localize from '../interface/localize';

export default localize('react-ui', {
  LOGIN_TITLE: 'Welcome',
  LOGIN_SUBTITLE: 'Login to continue',
  SIGNIN_TITLE: 'Sign in to Astrum Repository',
  SIGNIN_BUTTON: 'Sign in',
  SIGNIN_BUTTON_LOADING: 'Signing in...',
  SIGNIN_FOOTER_LEAD: 'New to Astrum Repository?',
  SSO_BUTTON: 'Continue with SSO',
  SSO_BUTTON_CLOUD: 'Login',
  SSO_BUTTON_LOADING: 'Redirecting...',
  SSO_DIVIDER_LABEL: 'or',
  USERNAME_LABEL: 'Username',
  PASSWORD_LABEL: 'Password',
  LOGIN_BUTTON: 'Log in',
  LOGIN_BUTTON_LOADING: 'Logging in...',
  CONTINUE_WITHOUT_LOGIN: 'Continue without login',
  INITIAL_PASSWORD_MESSAGE: 'To log in for the first time, use the generated admin password located at:',
  ERRORS: {
    WRONG_CREDENTIALS: 'Invalid username or password',
    CONNECTION_FAILED: 'Connection failed. Please check your network connection.',
    AUTHENTICATION_FAILED: 'An error occurred during authentication',
    USERNAME_REQUIRED: 'Username is required',
    PASSWORD_REQUIRED: 'Password is required',
    RATE_LIMITED: (seconds) =>
      seconds != null
        ? `Too many failed login attempts. Please wait ${seconds} second${seconds === 1 ? '' : 's'} before trying again.`
        : `Too many failed login attempts. Please try again later.`
  }
});
