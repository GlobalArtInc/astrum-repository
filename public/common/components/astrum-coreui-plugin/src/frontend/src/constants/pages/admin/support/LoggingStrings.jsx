
import React from 'react';
import { faScroll } from '@fortawesome/free-solid-svg-icons';

export default {
  LOGGING: {
    MENU: {
      text: 'Logging',
      description: 'Control logging levels',
      icon: faScroll
    },

    CREATE_BUTTON: 'Create Logger',
    RESET_ALL_BUTTON: 'Reset to Default Levels',
    RESET_BUTTON: 'Reset to Default Level',

    FILTER_PLACEHOLDER: 'Filter by Logger Name',

    NAME_LABEL: 'Logger Name',
    LEVEL_LABEL: 'Logger Level',

    MESSAGES: {
      SAVE_ERROR: 'An error occurred while saving the logger',
      RESETTING: 'Resetting to Default Level...',
      RESET_ERROR: 'An error occurred while resetting all loggers, see console for more details'
    },

    CONFIRM_UPDATE: {
      TITLE: 'Confirm Update?',
      MESSAGE: ({name, level}) => `Logger ${name} is already configured. Would you like to update its level to "${level}"?`,
      CONFIRM_BUTTON: 'Update Logger Level'
    },

    CONFIRM_RESET_ALL: {
      TITLE: 'Confirm Reset?',
      MESSAGE: 'Reset all loggers to their default levels?',
      CONFIRM_BUTTON: 'Reset All Loggers'
    },

    CONFIRM_RESET: {
      TITLE: 'Confirm Reset?',
      MESSAGE: 'Reset this logger to its default level? If this is a custom logger it will be removed.',
      CONFIRM_BUTTON: 'Reset Logger'
    }
  }
};