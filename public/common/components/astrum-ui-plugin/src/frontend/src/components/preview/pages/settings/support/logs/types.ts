import LogsStrings from '../../../../constants/pages/settings/support/LogsStrings';


/**
 * Represents a log file in the system
 */
export interface LogFile {
  fileName: string;
  size: number;
  lastModified: number;
}

/**
 * Refresh rate options for log viewer (in seconds)
 */
export const REFRESH_RATES = [
  { value: 0, label: LogsStrings.TYPES.MANUAL },
  { value: 20, label: LogsStrings.TYPES.EVERY_20_SECONDS },
  { value: 60, label: LogsStrings.TYPES.EVERY_MINUTE },
  { value: 120, label: LogsStrings.TYPES.EVERY_2_MINUTES },
  { value: 300, label: LogsStrings.TYPES.EVERY_5_MINUTES },
] as const;

/**
 * Log view size options (in KB)
 */
export const LOG_SIZES = [
  { value: 25, label: LogsStrings.TYPES.LAST_25KB },
  { value: 50, label: LogsStrings.TYPES.LAST_50KB },
  { value: 100, label: LogsStrings.TYPES.LAST_100KB },
] as const;

/**
 * Sort field options for log list
 */
export type LogSortField = 'fileName' | 'size' | 'lastModified';

/**
 * Sort direction
 */
export type SortDirection = 'asc' | 'desc';

/**
 * API URLs for logs
 */
const encodeLogFilename = (filename: string) => filename.split('/').map(encodeURIComponent).join('/');

export const LOGS_API = {
  LIST: '/service/rest/internal/logging/logs',
  VIEW: (filename: string) => `/service/rest/internal/logging/logs/${encodeLogFilename(filename)}`,
  MARK: '/service/rest/internal/logging/log/mark',
} as const;

