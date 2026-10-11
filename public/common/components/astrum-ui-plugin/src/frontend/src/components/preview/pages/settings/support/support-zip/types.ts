import SupportZipStrings from '../../../../constants/pages/settings/support/SupportZipStrings';


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
  { value: 0, label: SupportZipStrings.TYPES.NONE },
  { value: 1, label: SupportZipStrings.TYPES.ONE_DAY },
  { value: 2, label: SupportZipStrings.TYPES.TWO_DAYS },
  { value: 3, label: SupportZipStrings.TYPES.THREE_DAYS },
] as const;

/**
 * API URLs for support ZIP
 */
export const SUPPORT_ZIP_API = {
  CREATE: '/service/rest/v1/support/supportzippath',
  DOWNLOAD: (filename: string) => `service/rest/wonderland/download/${filename}`,
} as const;
