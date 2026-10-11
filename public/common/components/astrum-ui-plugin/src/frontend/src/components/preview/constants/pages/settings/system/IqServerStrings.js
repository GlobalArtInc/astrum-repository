import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  UTILS: {
    IQ_SERVER_URL_REQUIRED: 'IQ Server URL is required',
    PLEASE_ENTER_VALID_URL: 'Please enter a valid URL',
    AUTHENTICATION_METHOD_REQUIRED: 'Authentication method is required',
    USERNAME_REQUIRED: 'Username is required',
    PASSWORD_REQUIRED: 'Password is required',
    PASSWORD_REQUIRED_WHEN_CHANGING_URL: 'Password is required when changing the URL',
    TIMEOUT_MUST_BETWEEN_1_3600: 'Timeout must be between 1 and 3600 seconds',
    CANNOT_REACH_CONNECTION_REFUSED_CHECK: (target) => `Cannot reach ${target} — connection refused. Check the URL and that the IQ Server is running.`,
    CANNOT_REACH_CONNECTION_TIMED_OUT: (target) => `Cannot reach ${target} — connection timed out. Check the URL, network, and firewall.`,
    CANNOT_REACH: (target, cause) => `Cannot reach ${target} — ${cause}.`,
    HOST_NOT_FOUND_CHECK_URL: (host) => `Host not found: ${host}. Check the URL is spelled correctly.`,
    HOST_NOT_FOUND_CHECK_URL_2: 'Host not found. Check the URL is spelled correctly.',
    SSL_CERTIFICATE_COULD_NOT_VERIFIED: 'SSL certificate could not be verified. Add the IQ Server certificate to the Astrum truststore or use "Use Astrum Truststore".',
    AUTHENTICATION_FAILED_CHECK_USERNAME_PASSWORD: 'Authentication failed. Check the username and password.',
    IQ_SERVER_USER_DOES_NOT: 'The IQ Server user does not have permission to read policy data.',
  },
  TYPES: {
    DAYS_30: '30 Days',
    DAYS_60: '60 Days',
    DAYS_90: '90 Days',
    ALL_MONITORING: 'All monitoring',
    ENABLED: 'Enabled',
    DISABLED: 'Disabled',
  },
  EVALUATION_HOOK: {
    FAILED_LOAD_REPOSITORIES: 'Failed to load repositories',
    SETTINGS_SAVED: 'Settings saved',
    FAILED_SAVE_SETTINGS: 'Failed to save settings',
    SELECTIONS_UPDATED: 'Selections updated',
    FAILED_UPDATE_SELECTIONS: 'Failed to update selections',
  },
}, 'PREVIEW.IQ_SERVER');
