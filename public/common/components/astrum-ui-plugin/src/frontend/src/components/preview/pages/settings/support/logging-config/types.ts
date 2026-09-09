

/**
 * Represents a logger configuration
 */
export interface Logger {
  name: string;
  level: LogLevel;
  override?: boolean;
}

/**
 * Available log levels
 */
export type LogLevel = 'OFF' | 'ERROR' | 'WARN' | 'INFO' | 'DEBUG' | 'TRACE' | 'DEFAULT';

/**
 * Log levels in order of verbosity
 */
export const LOG_LEVELS: LogLevel[] = ['OFF', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'];

/**
 * Form data for creating/editing a logger
 */
export interface LoggerFormData {
  name: string;
  level: LogLevel;
}

/**
 * Sort field options for logger list
 */
export type LoggerSortField = 'name' | 'level';

/**
 * Sort direction
 */
export type SortDirection = 'asc' | 'desc';

/**
 * API URLs for logging configuration
 */
export const LOGGING_CONFIG_API = {
  LIST: '/service/rest/internal/ui/loggingConfiguration',
  GET: (name: string) => `/service/rest/internal/ui/loggingConfiguration/${encodeURIComponent(name)}`,
  UPDATE: (name: string) => `/service/rest/internal/ui/loggingConfiguration/${encodeURIComponent(name)}`,
  RESET: (name: string) => `/service/rest/internal/ui/loggingConfiguration/${encodeURIComponent(name)}/reset`,
  RESET_ALL: '/service/rest/internal/ui/loggingConfiguration/reset',
} as const;


