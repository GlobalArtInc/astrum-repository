import localize from '../../../interface/localize';

/**
 * Texts that Astrum One shows in the same sense on many pages: actions, generic labels and messages.
 * Texts whose translation depends on what they describe (e.g. "Enabled" of a task or of a repository) belong to the
 * strings of their page.
 */
export default localize('react-ui', {
  ACTIONS: {
    BACK: 'Back',
    BACK_TO_LIST: 'Back to List',
    CANCEL: 'Cancel',
    CLEAR_FILTERS: 'Clear Filters',
    CLOSE: 'Close',
    CONFIGURE: 'Configure',
    CONFIRM: 'Confirm',
    CREATE: 'Create',
    DELETE: 'Delete',
    DOWNLOAD: 'Download',
    EDIT: 'Edit',
    FILTER: 'Filter',
    LEARN_MORE: 'Learn more',
    LEARN_MORE_TITLE: 'Learn More',
    REFRESH: 'Refresh',
    RETRY: 'Retry',
    SAVE: 'Save',
    VIEW_DOCUMENTATION: 'View Documentation',
  },
  LABELS: {
    ACTIONS: 'Actions',
    CONFIGURE_STEP: 'Configure',
    DESCRIPTION: 'Description',
    FORMAT: 'Format',
    NAME: 'Name',
    SETTINGS: 'Settings',
    STATUS: 'Status',
    TYPE: 'Type',
  },
  MESSAGES: {
    EXAMPLE: (example) => `e.g., ${example}`,
    LOADING: 'Loading...',
    LOADING_FORM: 'Loading form...',
    NAME_REQUIRED: 'Name is required',
    OPERATION_FAILED: 'Operation failed',
    UNKNOWN_ERROR: 'Unknown error',
  },
}, 'PREVIEW.COMMON');
