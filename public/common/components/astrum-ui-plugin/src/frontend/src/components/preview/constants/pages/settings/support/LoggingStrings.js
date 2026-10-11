import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  FORM: {
    OVERRIDES_INHERITED_LEVEL: 'Setting a level here overrides the inherited level from the parent logger.',
    DELETE_TO_REVERT: 'Use "Delete" to revert to the inherited level.',
    LOADING_LOGGER: 'Loading logger...',
    CREATE_LOGGER: 'Create Logger',
    LOGGER_NAME: 'Logger Name',
    FULLY_QUALIFIED_LOGGER_NAME_E: 'Fully qualified logger name (e.g., org.globalart.astrum.repository)',
    LOGGER_LEVEL: 'Logger Level',
    LOG_LEVEL_ERROR_LEAST_VERBOSE: 'Log level: ERROR (least verbose) through TRACE (most verbose)',
  },
  FORM_MACHINE: {
    FAILED_LOAD_LOGGER: 'Failed to load logger',
    FAILED_SAVE_LOGGER: 'Failed to save logger',
  },
  LIST: {
    LOADING_LOGGERS: 'Loading loggers...',
    FILTER_LOGGER_NAME_PLACEHOLDER: 'Filter by logger name',
    SHOWING_LOGGERS: (filteredLoggersCount, loggersCount) => `Showing ${filteredLoggersCount} of ${loggersCount} loggers`,
    LOGGER_NAME: 'Logger Name',
    LOGGER_LEVEL: 'Logger Level',
    NO_LOGGERS_MATCH_FILTERS: 'No loggers match your filters',
    NO_LOGGERS_FOUND: 'No loggers found',
  },
  LIST_MACHINE: {
    FAILED_LOAD_LOGGERS: 'Failed to load loggers',
  },
  MACHINE: {
    ERROR_OCCURRED: 'An error occurred',
    NO_LOGGER_SELECTED_DELETION: 'No logger selected for deletion',
    FAILED_DELETE_LOGGER_OVERRIDE: 'Failed to delete logger override',
    FAILED_RESET_ALL_LOGGERS: 'Failed to reset all loggers',
  },
  PAGE: {
    LOGGING: 'Logging',
    RESET_DEFAULT_LEVELS: 'Reset to Default Levels',
    CREATE_LOGGER: 'Create Logger',
    CONTROL_LOGGING_LEVELS: 'Control logging levels',
    EDIT_LOGGER: 'Edit Logger',
    LOGGER: 'Logger',
    ABOUT_LOGGING_CONFIGURATION: 'About Logging Configuration',
    CONFIGURE_LOGGING_LEVELS_DIFFERENT_PACKAGES: 'Configure logging levels for different packages and classes in Astrum Repository. Increasing the logging level can help diagnose issues but may impact performance. Reset to defaults will restore all loggers to their original levels.',
    DELETE_LOGGER_OVERRIDE: 'Delete Logger Override',
    REMOVE_CUSTOM_LOG_LEVEL_LOGGER: (selectedLogger) => `Remove the custom log level for "${selectedLogger}"? The logger will inherit its level from the parent logger configuration. This effectively removes it from the custom loggers list.`,
    DELETING: 'Deleting...',
    RESET_ALL_LOGGERS: 'Reset All Loggers',
    YOU_SURE_YOU_WANT_RESET: 'Are you sure you want to reset all loggers to their default levels? This action cannot be undone.',
    RESET_ALL: 'Reset All',
  },
  FORM_HOOK: {
    LOGGER_CREATED_SUCCESSFULLY: (ctxName) => `Logger "${ctxName}" created successfully`,
    LOGGER_UPDATED: (ctxName, ctxLevel) => `Logger "${ctxName}" updated to ${ctxLevel}`,
  },
  LIST_HOOK: {
    LOG_LEVEL: 'Log Level',
  },
  HOOK: {
    LOGGER_OVERRIDE_REMOVED: (loggerName) => `Logger override removed for "${loggerName}"`,
    ALL_LOGGERS_RESET_DEFAULT_LEVELS: 'All loggers reset to default levels',
  },
  API: {
    FAILED_LOAD_LOGGERS: 'Failed to load loggers',
    FAILED_LOAD_LOGGER: 'Failed to load logger',
    FAILED_UPDATE_LOGGER: 'Failed to update logger',
    FAILED_RESET_LOGGER: 'Failed to reset logger',
    FAILED_RESET_ALL_LOGGERS: 'Failed to reset all loggers',
  },
}, 'PREVIEW.LOGGING');
