export default {
  SHELL: {
    HEADER: {
      EXPAND_SIDEBAR: 'Expand Sidebar',
      COLLAPSE_SIDEBAR: 'Collapse Sidebar',
      OPEN_MENU: 'Open menu',
      CLOSE_MENU: 'Close menu',
      OPEN_SEARCH: 'Search',
      CLOSE_SEARCH: 'Close search',
      HOME: 'Home',
      LOGO_ALT: 'Astrum Repository',
      SWITCH_TO_CLASSIC: 'Switch to Classic UI',
      SWITCH_TO_PREVIEW: 'Switch to Astrum One UI',
      SWITCH_FEEDBACK: {
        TITLE: 'Trying the new Astrum One UI',
        DESCRIPTION: 'Prefer the classic experience? Let us know why you\'re switching back.',
        PLACEHOLDER: 'Share your feedback...',
        LABEL: 'Optional feedback',
        TOO_LONG: (max, over) => `Feedback must be ${max} characters or less (${over} characters too many)`,
        USE_CLASSIC: 'Use Classic UI'
      },
      REFRESH: 'Refresh',
      CLASSIC_UNAVAILABLE: {
        TITLE: 'Not Available in Classic UI',
        TEXT: 'This view is only available in the Astrum One UI. The Classic UI doesn\'t have an equivalent page for this feature.',
        STAY: 'Stay on Astrum One UI',
        GO: 'Go to Classic Dashboard'
      },
      LANGUAGE: 'Language',
      SYSTEM_STATUS: 'System Status',
      SYSTEM_STATUS_UNHEALTHY: 'System Status: health checks are failing',
      SEARCH_LABEL: 'Search components',
      SEARCH_PLACEHOLDER: 'Search components or CVEs...',
      SEARCH_SHORTCUT: 'Ctrl K',
      SEARCH_SHORTCUT_MAC: '⌘ K',
      SEARCH_CLEAR: 'Clear search',
      SEARCH_PANEL: {
        LABEL: 'Search suggestions',
        RECENT: 'Recent searches',
        REMOVE_RECENT: (query) => `Remove “${query}” from recent searches`,
        EMPTY_HINT: 'Find components by name, group or version, or open a repository by its name.',
        SEARCH_IN: (query) => `Search for “${query}” in…`,
        THIS_REPOSITORY: 'this repository',
        ALL_REPOSITORIES: 'all repositories',
        REPOSITORIES: 'Repositories',
        COMPONENTS: 'Components',
        NO_RESULTS: 'No results',
        SEARCHING: 'Searching…',
        OPEN_HINT: '↵'
      }
    },
    THEME: {
      SWITCH_TO_DARK: 'Switch to dark mode',
      SWITCH_TO_LIGHT: 'Switch to light mode'
    },
    HELP: {
      TITLE: 'Help & Documentation',
      DOCUMENTATION: 'Documentation',
      RELEASE_NOTES: 'Release Notes',
      KNOWLEDGE_BASE: 'Knowledge Base',
      GUIDES: 'GlobalArt Guides',
      COMMUNITY: 'Community',
      ISSUE_TRACKER: 'Issue Tracker',
      VERSION: (version) => `Version: ${version}`
    },
    ACCOUNT: {
      LOG_IN: 'Log In',
      MY_ACCOUNT: 'My Account',
      ACCESS_TOKENS: 'Access Tokens',
      NUGET_API_KEY: 'NuGet API Key',
      USER_TOKEN: 'User Token',
      LOG_OUT: 'Log Out'
    },
    NAVIGATION: {
      DASHBOARD: 'Dashboard',
      SEARCH: 'Search',
      CUSTOM_SEARCH: 'Custom',
      BROWSE: 'Browse',
      UPLOAD: 'Upload',
      TAGS: 'Tags',
      API: 'API',
      SETTINGS: 'Settings',
      MAIN_MENU: 'Main menu',
      PRODUCT_VERSION: (version) => `Astrum Repository ${version}`
    },
    LOGIN: {
      HOME_LINK: 'GlobalArt Astrum Repository home'
    },
    MISSING_ROUTE: {
      TITLE: 'RESOURCE NOT FOUND',
      TEXT: 'This resource is not available. It may have been moved, deleted, or you don’t have access.',
      SUGGESTIONS: 'You might want to:',
      DASHBOARD: 'Return to Dashboard',
      DOCUMENTATION: 'Visit Documentation'
    },
    SESSION_EXPIRY: {
      TITLE: 'Session Expired',
      SUBTITLE: 'Authentication Required',
      REFRESH: 'Refresh Page',
      SIGN_IN: 'Sign In'
    },
    EXTJS_CONTAINER: {
      FAILED_TITLE: 'Failed to Load Classic UI Component',
      FAILED_TEXT: 'The legacy ExtJS framework failed to initialize. This component requires ExtJS to function.',
      ERROR: (message) => `Error: ${message}`,
      RELOAD: 'Reload Page',
      LOADING: 'Loading Classic UI component...'
    },
    SELECT: {
      CLEAR: 'Clear selection',
      SEARCH: 'Search options',
      OPTIONS: 'Options',
      NO_MATCHES: 'No matches found'
    },
    TABLE: {
      LOADING: 'Loading...',
      EMPTY: 'No data available'
    },
    FILTERS: {
      CLEAR_ALL_TITLE: 'Clear all filters',
      CLEAR_ALL: 'Clear all'
    },
    LOADING: 'Loading',
    BREADCRUMB: 'Breadcrumb'
  }
};
