import React from 'react';
import localize from '../../../interface/localize';

export default localize('react-ui', {
  COMING_SOON: {
    THIS_FEATURE: 'This Feature',
    WERE_WORKING_HARD_BRING_FEATURE: 'We\'re working hard to bring this feature to the new Astrum One UI.',
    COMING_SOON: 'Coming Soon',
    PAGE_PART_ASTRUM_ONE_UI: 'This page is part of the Astrum One UI rollout. The feature will be available soon with a fresh new look using Radix UI components.',
    GO_BACK: 'Go Back',
    GO_DASHBOARD: 'Go to Dashboard',
  },
  DEEP_RESEARCH: {
    RESEARCH_GUIDE_ARIA: 'Research in Guide',
    RESEARCH_GUIDE: 'Research in Guide',
    RESEARCH_COMPONENT_GLOBALART_GUIDE: 'Research this component in GlobalArt Guide',
  },
  ENTITY_TABLE: {
    DATA_TABLE: 'Data table',
    NO_DATA_AVAILABLE: 'No data available',
    VIEW: (key) => `View ${key}`,
  },
  ERROR_STATE: {
    SOMETHING_WENT_WRONG: 'Something went wrong',
    TRY_AGAIN: 'Try again',
  },
  FILTER_SIDEBAR: {
    TYPE_FILTER: 'Type to filter...',
    FILTERS: 'Filters',
    FILTERS_APPLY_AUTOMATICALLY: 'Filters apply automatically',
    CLEAR_ALL_FILTERS: 'Clear all filters',
    CLEAR_ALL: 'Clear all',
  },
  FIREWALL_NOT_SUPPORTED: {
    FORMAT_NOT_SUPPORTED: 'Format not Supported',
    FIREWALL_NOT_AVAILABLE: 'Firewall Not Available',
    REPOSITORY_FIREWALL_DOES_NOT_SUPPORT: (value) => `Repository Firewall does not support the ${value} format. No malware scanning or component quarantine is available for this repository.`,
    LEARN_ABOUT_SUPPORTED_FORMATS: 'Learn about supported formats',
    FIREWALL_NOT_SUPPORTED: (value) => `Firewall: Not supported for ${value}`,
    NOT_SUPPORTED: 'Not supported',
  },
  MOBILE_FILTER_DRAWER: {
    CLEAR_ALL: 'Clear all',
  },
  PAGE_HEADER: {
    BREADCRUMB_ARIA: 'Breadcrumb',
  },
  SEARCH: {
    SEARCH_COMPONENTS_CVES_PLACEHOLDER: 'Search components or CVEs...',
    PRESS_ENTER_SEARCH_ALL_RESULTS: 'Press Enter to search all results',
  },
  SESSION_EXPIRY: {
    SESSION_HAS_EXPIRED_PLEASE_SIGN: 'Your session has expired. Please sign in again to continue.',
    SESSION_EXPIRED: 'Session Expired',
    AUTHENTICATION_REQUIRED: 'Authentication Required',
    REFRESH_PAGE: 'Refresh Page',
    SIGN_IN: 'Sign In',
  },
  SETTINGS_LAYOUT: {
    BACK_SETTINGS: '← Back to Settings',
  },
  SPARKLINE: {
    TREND: (trend) => `Trend ${trend}`,
  },
  STATUS_BADGE: {
    STATUS: (displayLabel) => `Status: ${displayLabel}`,
  },
  STICKY_ACTION_BAR: {
    DISCARD: 'Discard',
    UNSAVED_CHANGES: 'Unsaved changes',
  },
  RECOVERY_MODE_ALERT: {
    RECOVERY_MODE_ENABLED: 'Recovery Mode Enabled',
    WHILE_RECOVERY_MODE_DATA_REPAIR: 'While Recovery Mode is on, data repair conflicting tasks are blocked to protect data consistency.',
    VIEW_DETAILS: 'View Details',
  },
  SYSTEM_ALERT: {
    DISMISS_ALERT_ARIA: 'Dismiss alert',
  },
  SYSTEM_STATUS_BELL: {
    SYSTEM_STATUS: 'System Status',
  },
  TABLE_PAGINATION: {
    JUMP_PAGE_ARIA: 'Jump to page',
    NO_PAGE_STARTS: (draft) => `No page starts with "${draft}"`,
    SHOWING: 'Showing',
    OF_ITEMS: (localeString, value) => `of ${localeString}${value}`,
    PAGE: 'Page',
    OF_PAGES: (totalPages) => `of ${totalPages}`,
    PAGINATION_PAGE: (currentPage, totalPages) => `Pagination, page ${currentPage} of ${totalPages}`,
    FIRST_PAGE_ARIA: 'First page',
    PREVIOUS_PAGE_ARIA: 'Previous page',
    LOAD_MORE: 'Load more',
    NEXT_PAGE: 'Next page',
    LAST_PAGE_ARIA: 'Last page',
  },
  THEME_SWITCHER: {
    SWITCH_LIGHT_MODE: 'Switch to light mode',
    SWITCH_DARK_MODE: 'Switch to dark mode',
  },
  SETTINGS_ALERT: {
    DISMISS_ARIA: 'Dismiss',
  },
  SETTINGS_COMBOBOX: {
    SEARCH: 'Search...',
    SELECT_TYPE: 'Select or type...',
    SELECTED: (selectedValuesCount) => `${selectedValuesCount} selected`,
    MATCHING: (shown, total) => `${shown} of ${total} matching`,
    SEARCHING: 'Searching...',
    NO_RESULTS_MATCHING: (inputValue) => `No results matching “${inputValue}”`,
    REMOVE: (fullLabel) => `Remove ${fullLabel}`,
    MORE: (overflowCount) => `+${overflowCount} more`,
  },
  SETTINGS_FORM: {
    DISCARD: 'Discard',
    UNSAVED_CHANGES: 'Unsaved changes',
    UNSAVED_CHANGES_TITLE: 'Unsaved Changes',
    YOU_HAVE_UNSAVED_CHANGES_YOU: 'You have unsaved changes. Are you sure you want to leave? Your changes will be lost.',
    STAY: 'Stay',
    LEAVE: 'Leave',
  },
  SETTINGS_PASSWORD_INPUT: {
    HIDE_PASSWORD: 'Hide password',
    SHOW_PASSWORD: 'Show password',
  },
  SETTINGS_SELECT: {
    SELECT_OPTION: 'Select an option',
  },
  SETTINGS_TRANSFER_LIST: {
    AVAILABLE: 'Available',
    SELECTED: 'Selected',
    FILTER_PLACEHOLDER: 'Filter...',
    NO_MATCHES: 'No matches',
    NO_ITEMS_AVAILABLE: 'No items available',
    MOVE_ALL_SELECTED_ARIA: 'Move all to selected',
    MOVE_ALL: 'Move all',
    MOVE_SELECTED_RIGHT_ARIA: 'Move selected to right',
    MOVE_SELECTED: 'Move selected',
    MOVE_SELECTED_LEFT_ARIA: 'Move selected to left',
    REMOVE_SELECTED: 'Remove selected',
    MOVE_ALL_AVAILABLE_ARIA: 'Move all to available',
    REMOVE_ALL: 'Remove all',
    ALL_ITEMS: 'All items',
    NO_ITEMS_SELECTED: 'No items selected',
  },
  WIZARD_FORM: {
    CONTINUE: 'Continue',
  },
  BLOB_STORE_INFO: {
    UNKNOWN: 'Unknown',
    FAILED_FETCH_BLOB_STORE: 'Failed to fetch blob store',
  },
  REPOSITORY_METRICS: {
    FAILED_FETCH_REPOSITORY_METRICS: 'Failed to fetch repository metrics',
  },
  DELETE_CONFIRMATION: {
    DELETE: (entityType) => `Delete ${entityType}?`,
    ACTION_CANNOT_UNDONE_TYPED: (entityType) => `This action cannot be undone. All data associated with this ${entityType} will be permanently deleted.`,
    ACTION_CANNOT_UNDONE_NAMED: (name) => <>This action cannot be undone. All data associated with {name} will be permanently deleted.</>,
    ENTITIES: {
      BLOB_STORE: {
        TITLE: 'Delete blob store?',
        MESSAGE: 'This action cannot be undone. All data associated with this blob store will be permanently deleted.',
      },
      REPOSITORY: {
        TITLE: 'Delete repository?',
        MESSAGE: 'This action cannot be undone. All data associated with this repository will be permanently deleted.',
      },
      ROUTING_RULE: {
        TITLE: 'Delete routing rule?',
        MESSAGE: 'This action cannot be undone. All data associated with this routing rule will be permanently deleted.',
      },
      TASK: {
        TITLE: 'Delete task?',
        MESSAGE: 'This action cannot be undone. All data associated with this task will be permanently deleted.',
      },
      USER: {
        TITLE: 'Delete user?',
        MESSAGE: 'This action cannot be undone. All data associated with this user will be permanently deleted.',
      },
    },
    CONFIRMATION_WORD: 'Delete',
    TYPE_CONFIRM_DELETION: 'Type this to confirm deletion',
    ACKNOWLEDGEMENT: 'Acknowledgement',
    TYPE_CONFIRM: (CONFIRMATION_TEXT) => `Type "${CONFIRMATION_TEXT}" to confirm`,
    ACKNOWLEDGEMENT_ARIA: 'Acknowledgement',
    CONFIRMATION_TEXT_PROVIDED_INCORRECT: 'The confirmation text provided is incorrect',
    DELETING: 'Deleting...',
  },
  LOADING_STATE: {
    LOADING: 'Loading',
    LOADING_MESSAGE: (message) => `Loading: ${message}`,
  },
}, 'PREVIEW.SHARED');
