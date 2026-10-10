export default {
  SETTINGS_HUB: {
    TITLE: 'Settings',
    DESCRIPTION: 'Configure security, repositories, system, and support',
    SECURITY: {
      TITLE: 'Security',
      USERS: {TEXT: 'Users', DESCRIPTION: 'Manage user accounts and access'},
      ROLES: {TEXT: 'Roles', DESCRIPTION: 'Define user permissions and access control'},
      PRIVILEGES: {TEXT: 'Privileges', DESCRIPTION: 'Configure granular permission settings'},
      REALMS: {TEXT: 'Realms', DESCRIPTION: 'Configure authentication sources and order'},
      LDAP: {TEXT: 'LDAP', DESCRIPTION: 'Configure LDAP integration'},
      SAML: {TEXT: 'SAML', DESCRIPTION: 'Configure SAML single sign-on'},
      OAUTH2: {TEXT: 'OAuth2', DESCRIPTION: 'Configure OAuth 2.0 authentication'},
      ATLASSIAN_CROWD: {TEXT: 'Atlassian Crowd', DESCRIPTION: 'Configure Atlassian Crowd integration'},
      ANONYMOUS_ACCESS: {TEXT: 'Anonymous Access', DESCRIPTION: 'Configure anonymous access settings'},
      SSL_CERTIFICATES: {TEXT: 'SSL Certificates', DESCRIPTION: 'Manage trusted SSL certificates'},
      USER_TOKENS: {TEXT: 'User Tokens', DESCRIPTION: 'Manage API access tokens'}
    },
    REPOSITORY: {
      TITLE: 'Repository',
      REPOSITORIES: {TEXT: 'Repositories', DESCRIPTION: 'Manage Maven, npm, Docker, and other repositories'},
      BLOB_STORES: {TEXT: 'Blob Stores', DESCRIPTION: 'Configure local and cloud blob storage'},
      CLEANUP_POLICIES: {TEXT: 'Cleanup Policies', DESCRIPTION: 'Configure automated component cleanup'},
      ROUTING_RULES: {TEXT: 'Routing Rules', DESCRIPTION: 'Control component access by path patterns'},
      CONTENT_SELECTORS: {TEXT: 'Content Selectors', DESCRIPTION: 'Define content filtering expressions'},
      DATA_STORE: {TEXT: 'Data Store', DESCRIPTION: 'Configure repository metadata database'},
      PROPRIETARY: {TEXT: 'Proprietary', DESCRIPTION: 'Manage proprietary component settings'}
    },
    SYSTEM: {
      TITLE: 'System',
      TASKS: {TEXT: 'Tasks', DESCRIPTION: 'Schedule and monitor system tasks'},
      EMAIL_SERVER: {TEXT: 'Email Server', DESCRIPTION: 'Configure email server and notifications'},
      HTTP: {TEXT: 'HTTP', DESCRIPTION: 'Configure HTTP and HTTPS settings'},
      API: {TEXT: 'API', DESCRIPTION: 'Configure REST API settings'},
      CAPABILITIES: {TEXT: 'Capabilities', DESCRIPTION: 'Configure system capabilities and integrations'},
      NODES: {TEXT: 'Nodes', DESCRIPTION: 'View cluster nodes and status'},
      ASTRUM_ONE_UI: {TEXT: 'Astrum One UI', DESCRIPTION: 'Configure Astrum One UI settings and features'}
    },
    SUPPORT: {
      TITLE: 'Support',
      SYSTEM_INFORMATION: {TEXT: 'System Information', DESCRIPTION: 'View system information and diagnostics'},
      LOGS: {TEXT: 'Logs', DESCRIPTION: 'View application logs'},
      LOGGING_CONFIGURATION: {TEXT: 'Logging Configuration', DESCRIPTION: 'Configure log levels and output'},
      METRIC_HEALTH: {TEXT: 'Metric Health', DESCRIPTION: 'View system metrics and health checks'},
      SUPPORT_ZIP: {TEXT: 'Support ZIP', DESCRIPTION: 'Generate support diagnostics bundle'},
      SUPPORT_REQUEST: {TEXT: 'Support Request', DESCRIPTION: 'Submit a support request'}
    }
  }
};
