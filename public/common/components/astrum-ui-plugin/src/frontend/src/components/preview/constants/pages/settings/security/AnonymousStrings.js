import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  VALIDATION: {
    USERNAME_REQUIRED: 'Username is required',
    REALM_REQUIRED: 'Realm is required',
  },
  PAGE: {
    LOADING_ANONYMOUS_ACCESS_SETTINGS: 'Loading anonymous access settings...',
    ANONYMOUS_ACCESS: 'Anonymous Access',
    CONFIGURE_ANONYMOUS_USER_ACCESS_SETTINGS: 'Configure anonymous user access settings',
    ANONYMOUS: 'Anonymous',
    CURRENT_SETTINGS: 'Current Settings',
    ENABLED: 'Enabled',
    DISABLED: 'Disabled',
    USERNAME: 'Username',
    REALM: 'Realm',
    ANONYMOUS_ACCESS_CONFIGURATION: 'Anonymous Access Configuration',
    WHEN_ENABLED_ANONYMOUS_USERS_CAN: 'When enabled, anonymous users can access content without authentication based on the permissions granted to the anonymous user account.',
    ALLOW_ANONYMOUS_USERS_ACCESS_SERVER: 'Allow anonymous users to access the server',
    ENABLE_ANONYMOUS_ACCESS_REPOSITORY_CONTENT: 'Enable anonymous access to repository content',
    USERNAME_USED_ANONYMOUS_ACCESS: 'The username that will be used for anonymous access',
    SECURITY_REALM_USE_ANONYMOUS_AUTHENTICATION: 'The security realm to use for anonymous authentication',
    ABOUT_ANONYMOUS_ACCESS: 'About Anonymous Access',
    ANONYMOUS_ACCESS_ALLOWS_USERS_BROWSE: 'Anonymous access allows users to browse and access repository content without authenticating. The permissions granted to the anonymous user determine what content is accessible.',
    DISABLE_ANONYMOUS_ACCESS: 'Disable Anonymous Access',
    DISABLING_ANONYMOUS_ACCESS_REQUIRE_AUTHENTICATION: 'Disabling anonymous access will require authentication for all requests. Users will need to log in to access the repository. Are you sure you want to continue?',
  },
  MESSAGES: {
    ANONYMOUS_ACCESS_SETTINGS_SAVED_SUCCESSFULLY: 'Anonymous access settings saved successfully',
    ANONYMOUS_ACCESS_DISABLED: 'Anonymous access disabled',
  },
}, 'PREVIEW.ANONYMOUS');
