import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  PAGE: {
    RECOVERY_MODE: 'Recovery Mode',
    WHILE_RECOVERY_MODE_ENABLED_SYSTEM: 'While recovery mode is enabled, the system blocks task types that conflict with data repair checks or execution.',
    ENABLE: 'Enable',
    DISABLE: 'Disable',
    ENABLED: 'Enabled',
    DISABLED: 'Disabled',
    RECONCILIATION_TASKS_STILL_RUNNING_WAIT: 'Reconciliation tasks are still running. Wait for them to finish before disabling.',
    DISABLE_RECOVERY_MODE_UNEXECUTED_PLANS: 'Disable recovery mode with unexecuted plans?',
    ONE_MORE_DATA_REPAIR_PLANS: 'One or more data repair plans have not been executed. If Recovery Mode is disabled now, unexecuted plans may become outdated.',
    DISABLE_MODE: 'Disable Mode',
    RECOVERY_MODE_ENABLED: 'Recovery mode enabled',
    FAILED_ENABLE_RECOVERY_MODE: 'Failed to enable recovery mode',
    RECOVERY_MODE_DISABLED: 'Recovery mode disabled',
    FAILED_DISABLE_RECOVERY_MODE: 'Failed to disable recovery mode',
    LOADING_RECOVERY_MODE: 'Loading recovery mode…',
  },
  TASKS_TABLE: {
    SEARCH_TASKS_PLACEHOLDER: 'Search tasks…',
    SEARCH_TASKS_ARIA: 'Search tasks',
    LAST_RUN: 'Last run',
    LAST_RESULT: 'Last result',
    NO_DATA_REPAIR_TASKS: 'No data repair tasks',
    DATA_REPAIR_TASKS_APPEAR_HERE: 'Data repair tasks appear here when recovery mode creates reconciliation work.',
    NO_TASKS_FOUND: 'No tasks found',
    NO_TASKS_MATCH_TRY_DIFFERENT: (query) => `No tasks match "${query}". Try a different search term.`,
    CLEAR_SEARCH: 'Clear Search',
    OPEN_TASK: (name) => `Open task ${name}`,
  },
  API: {
    FAILED_LOAD_RECOVERY_MODE_SETTINGS: 'Failed to load recovery mode settings',
    FAILED_ENABLE_RECOVERY_MODE: 'Failed to enable recovery mode',
    FAILED_DISABLE_RECOVERY_MODE: 'Failed to disable recovery mode',
  },
}, 'PREVIEW.RECOVERY_MODE');
