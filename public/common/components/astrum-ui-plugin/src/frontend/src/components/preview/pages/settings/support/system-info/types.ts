import SystemInfoStrings from '../../../../constants/pages/settings/support/SystemInfoStrings';


/**
 * System Information types for Preview UI
 */

export interface SystemInfoSection {
  [key: string]: string | number | boolean | null | undefined;
}

export interface SystemInformation {
  'astrum-status'?: SystemInfoSection;
  'astrum-node'?: SystemInfoSection;
  'astrum-license'?: SystemInfoSection;
  'astrum-configuration'?: SystemInfoSection;
  'astrum-properties'?: SystemInfoSection;
  'system-time'?: SystemInfoSection;
  'system-properties'?: SystemInfoSection;
  'system-environment'?: SystemInfoSection;
  'system-runtime'?: SystemInfoSection;
  'system-network'?: SystemInfoSection;
  'system-filestores'?: SystemInfoSection;
  [key: string]: SystemInfoSection | undefined;
}

export interface HANode {
  nodeId: string;
  friendlyName?: string;
  hostname?: string;
  socketAddress?: string;
  local?: boolean;
}

export interface HASystemInformation {
  [nodeId: string]: SystemInformation;
}

export interface SystemInfoPageProps {
  className?: string;
}

export interface SystemInfoSectionProps {
  title: string;
  data: SystemInfoSection;
  /** Default open state for uncontrolled mode */
  defaultOpen?: boolean;
  /** Controlled open state (takes precedence over defaultOpen) */
  open?: boolean;
  /** Callback when section is toggled (for controlled mode) */
  onToggle?: (isOpen: boolean) => void;
  className?: string;
}

export interface NodeSelectorProps {
  nodes: HANode[];
  selectedNode: string | null;
  onNodeSelect: (nodeId: string) => void;
  className?: string;
}

/**
 * Format section title from key
 * e.g., 'astrum-status' -> 'Astrum Status'
 */
export function formatSectionTitle(key: string): string {
  return key
    .split('-')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

/**
 * Format value for display
 */
export function formatValue(value: string | number | boolean | null | undefined): string {
  if (value === null || value === undefined) {
    return '-';
  }
  if (typeof value === 'boolean') {
    return value ? SystemInfoStrings.TYPES.YES : SystemInfoStrings.TYPES.NO;
  }
  return String(value);
}


