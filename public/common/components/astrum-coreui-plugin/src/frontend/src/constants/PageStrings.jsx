export default {
  PAGES: {
    API: {
      SEARCH_LABEL: 'Search Operations',
      SEARCH_SUB_LABEL: 'Search by operation name, path, or description',
      SEARCH_PLACEHOLDER: 'Enter search term...',
      FILTERING: 'Filtering:',
      FILTERING_TEXT: (term) => `Operations containing "${term}"`,
      CLEAR: 'Clear'
    },
    GOOGLE_BLOB_STORE: {
      REGION_HELP: 'The region is automatically set based on where Astrum Repository is running in GCP. Ensure the bucket is in the same region.',
      ENCRYPTION_HELP: 'Data is encrypted by default. Enable KMS to use a custom encryption key.',
      ENCRYPTION_NOTE: 'Note: Encryption settings cannot be changed once the bucket is created'
    },
    REPOSITORY_STATUS: {
      ONLINE: 'Online',
      OFFLINE: 'Offline'
    },
    TOGGLEABLE_TEXT_INPUT: 'Toggle Text Input',
    SELECTION_MODAL: {
      TITLE: (title) => `${title} Selection`,
      FILTER: 'Filter',
      SELECT: 'Select',
      NAME: 'Name',
      DESCRIPTION: 'Description',
      SELECTED: (count) => `${count} Selected`,
      NO_PRIVILEGES: 'No privileges available',
      NO_ROLES: 'No roles available',
      CANCEL: 'Cancel',
      CONFIRM: 'Confirm'
    },
    ROLES_FORM: {
      APPLIED_PRIVILEGES: 'Applied Privileges',
      MODIFY_PRIVILEGES: 'Modify Applied Privileges',
      APPLIED_ROLES: 'Applied Roles',
      MODIFY_ROLES: 'Modify Applied Roles'
    },
    SAML_SIGNATURE: {
      DEFAULT: 'Default',
      TRUE: 'True',
      FALSE: 'False'
    },
    SUPPORT_ZIP: {
      LIMIT_NONE: 'None',
      LIMIT_1_DAY: '1 Day',
      LIMIT_2_DAYS: '2 Days',
      LIMIT_3_DAYS: '3 Days',
      CREATE: 'Create support ZIP',
      CREATE_ALL_NODES: 'Create support ZIP (all nodes)',
      CREATED: 'Support ZIP has been created.',
      CREATED_HELP: 'You can reference this file on the filesystem or download the file from your browser.'
    },
    UPLOAD: {
      SELECT: 'Select...',
      DELETE: 'Delete'
    },
    EULA_ONBOARDING: 'By clicking \'Agree\', you acknowledge that you have read and agree to the End User License Agreement.',
    UPGRADE_TRIGGER: {
      ACKNOWLEDGE: 'Acknowledge'
    }
  }
};
