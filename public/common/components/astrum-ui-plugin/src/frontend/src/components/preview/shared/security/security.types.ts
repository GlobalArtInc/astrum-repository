

/**
 * Shared security status data used by HealthCheck and Firewall components.
 */
export interface SecurityStatusData {
  repositoryName: string;
  affectedComponentCount: number;
  criticalComponentCount: number;
  severeComponentCount: number;
  moderateComponentCount: number;
  quarantinedComponentCount: number;
  reportUrl?: string;
  message?: string | null;
  errorMessage?: string | null;

  /** Rich Health Check: report metadata */
  reportDate?: string;
  reportAge?: string;
  /** Components identified in scan */
  componentsIdentified?: number;
  componentsTotal?: number;
  /** Security vulns by severity (Critical 7-10, Severe 4-6, Moderate 1-3) - when absent, use critical/severe/moderateComponentCount */
  securityCriticalCount?: number;
  securitySevereCount?: number;
  securityModerateCount?: number;
  /** Total license issues (when API only provides single count, no breakdown) */
  licenseIssueCount?: number;
  /** License warnings by type */
  licenseCopyleftCount?: number;
  licenseNonStandardCount?: number;
  licenseNotProvidedCount?: number;
  licenseWeakCopyleftCount?: number;
  licenseLiberalCount?: number;
  /** Threat level bar counts [1..10]; index 0 = threat 1 */
  threatLevelCounts?: number[];
}

/**
 * Minimal repository info required by security cells.
 */
export interface SecurityRepositoryInfo {
  name: string;
  type: string;
  format: string;
}
