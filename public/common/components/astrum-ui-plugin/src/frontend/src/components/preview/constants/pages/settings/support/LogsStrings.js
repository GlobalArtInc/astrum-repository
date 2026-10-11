import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  LIST: {
    LOADING_LOG_FILES: 'Loading log files...',
    FILTER_FILE_NAME_PLACEHOLDER: 'Filter by file name',
    FILE_NAME: 'File Name',
    SIZE: 'Size',
    LAST_MODIFIED: 'Last Modified',
    NO_LOG_FILES_MATCH_FILTER: 'No log files match your filter',
    NO_LOG_FILES_FOUND: 'No log files found',
  },
  LIST_MACHINE: {
    FAILED_LOAD_LOG_FILES: 'Failed to load log files',
  },
  PAGE: {
    LOGS: 'Logs',
    VIEW_CURRENT_LOG_CONTENTS: 'View the current log contents',
    LOG_VIEWER: 'Log Viewer',
    YOU_DO_NOT_HAVE_PERMISSION: 'You do not have permission to view logs.',
    LOGS_NOT_AVAILABLE_VIA_UI: 'Logs are not available via the UI in a clustered environment. Please access log files directly from the file system on each node.',
    ABOUT_LOGS: 'About Logs',
    VIEW_DOWNLOAD_LOG_FILES_ASTRUM: 'View and download log files from the Astrum Repository server. The main log file is astrum.log which contains application logs. You can insert markers into the log to help identify specific events.',
  },
  VIEWER: {
    VIEWING: (filename) => `Viewing ${filename}`,
    MARKER_INSERT: 'Marker to insert:',
    MARK_PLACEHOLDER: 'MARK',
    INSERT: 'Insert',
    REFRESH_RATE: 'Refresh Rate:',
    SIZE: 'Size:',
    LOADING_LOG_CONTENT: 'Loading log content...',
    LOG_CONTENT: (filename) => `Log content for ${filename}`,
  },
  VIEWER_MACHINE: {
    FAILED_LOAD_LOG_CONTENT: 'Failed to load log content',
    FAILED_INSERT_MARK: 'Failed to insert mark',
  },
  TYPES: {
    MANUAL: 'Manual',
    EVERY_20_SECONDS: 'Every 20 seconds',
    EVERY_MINUTE: 'Every minute',
    EVERY_2_MINUTES: 'Every 2 minutes',
    EVERY_5_MINUTES: 'Every 5 minutes',
    LAST_25KB: 'Last 25KB',
    LAST_50KB: 'Last 50KB',
    LAST_100KB: 'Last 100KB',
  },
  API: {
    FAILED_LOAD_LOG_FILES: 'Failed to load log files',
    FAILED_LOAD_LOG_CONTENT: 'Failed to load log content',
    FAILED_INSERT_MARK: 'Failed to insert mark',
  },
}, 'PREVIEW.LOGS');
