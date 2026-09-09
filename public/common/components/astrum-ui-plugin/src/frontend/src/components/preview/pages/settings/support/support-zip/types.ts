

/**
 * Parameters for creating a support ZIP
 */
export interface SupportZipParams {
  systemInformation: boolean;
  threadDump: boolean;
  configuration: boolean;
  security: boolean;
  log: boolean;
  taskLog: boolean;
  auditLog: boolean;
  metrics: boolean;
  jmx: boolean;
  replication: boolean;
  archivedLog: number;
  limitFileSizes: boolean;
  limitZipSize: boolean;
}

/**
 * Response from creating a support ZIP
 */
export interface SupportZipResponse {
  file: string;
  name: string;
  size: string;
  truncated: boolean;
}

export type SupportZipNodeStatus =
  | 'NODE_UNAVAILABLE'
  | 'NOT_CREATED'
  | 'CREATING'
  | 'COMPLETED'
  | 'FAILED';

/**
 * HA Node information returned by the supportzip internal endpoints.
 * Mirrors the backend SupportZipInfoData shape.
 */
export interface NodeInfo {
  nodeId: string;
  hostname: string;
  status: SupportZipNodeStatus;
  blobRef?: string;
  lastUpdated?: number;
}

/**
 * Default support ZIP parameters
 */
export const DEFAULT_SUPPORT_ZIP_PARAMS: SupportZipParams = {
  systemInformation: true,
  threadDump: true,
  configuration: true,
  security: true,
  log: true,
  taskLog: true,
  auditLog: true,
  metrics: true,
  jmx: true,
  replication: true,
  archivedLog: 0,
  limitFileSizes: true,
  limitZipSize: true,
};

/**
 * Archived log options
 */
export const ARCHIVED_LOG_OPTIONS = [
  { value: 0, label: 'None' },
  { value: 1, label: '1 Day' },
  { value: 2, label: '2 Days' },
  { value: 3, label: '3 Days' },
] as const;

/**
 * API URLs for support ZIP
 */
export const SUPPORT_ZIP_API = {
  CREATE: '/service/rest/v1/support/supportzippath',
  DOWNLOAD: (filename: string) => `service/rest/wonderland/download/${filename}`,
} as const;
