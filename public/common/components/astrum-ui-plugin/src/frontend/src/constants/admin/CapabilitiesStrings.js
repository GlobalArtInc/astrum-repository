

import { faServer } from '@fortawesome/free-solid-svg-icons';

export default {
  CAPABILITIES: {
    MENU: {
      text: 'Capabilities',
      description: 'Manage system capabilities for integrations and automation features.',
      icon: faServer,
    },
    LIST: {
      CREATE_BUTTON: 'Create Capability',
      EMPTY_LIST: 'No capabilities were found.',
      COLUMNS: {
        TYPE: 'Type',
        STATE: 'State',
        CATEGORY: 'Category',
        REPOSITORY: 'Repository',
        DESCRIPTION: 'Description',
        NOTES: 'Notes',
      },
    },
    CREATE: {
      TITLE: 'Create Capability',
      DESCRIPTION: 'Create system capabilities for integrations and automation features.',
      FORM: {
        TITLE: 'Select Capability',
        SELECT_TYPE: 'Select Type',
      },
    },
    EDIT: {
      FORM: {
        TITLE: 'Capability Settings',
        CAPABILITY_STATE: 'Capability State',
        ENABLED_LABEL: 'Enable',
        NOTES: 'Notes',
        NOTES_HELP_TEXT: 'Optional notes about configured capability',
      },
      LABELS: {
        TYPE: 'Type',
        DESCRIPTION: 'Description',
        STATE: 'State',
        CATEGORY: 'Category',
        STATUS: 'Status',
        ABOUT: 'About',
      },
    },
  },
};
