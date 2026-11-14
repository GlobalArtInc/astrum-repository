import { ApiError } from '../../../api';

export const SEVERITIES = ['critical', 'high', 'medium', 'low', 'unknown'] as const;

export type Severity = (typeof SEVERITIES)[number];

export interface SeverityCounts {
  critical: number;
  high: number;
  medium: number;
  low: number;
  unknown: number;
}

export interface ScanStatus {
  state: string;
  scannedAt: string | null;
  packages: number | null;
}

export interface PolicyRef {
  name: string;
  revision: number | null;
}

export interface Reason {
  kind: string;
  advisory?: string;
  package?: string;
  reason?: string;
  findings?: number;
  scan?: string;
  message?: string;
  omitted?: number;
}

export interface Verdict {
  policy: string;
  revision: number;
  state: string;
  blocked: boolean;
  counts: SeverityCounts;
  maxSeverity: string | null;
  reasons: Reason[];
  computedAt: string;
}

export interface Vulnerability {
  id: string;
  aliases: string[];
  severity: string;
  score: number | null;
  summary: string;
  fixed: string | null;
  malicious: boolean;
  excepted: boolean;
}

export interface ComponentSecurity {
  scan: ScanStatus;
  policy: PolicyRef | null;
  verdict: Verdict | null;
  vulnerabilities: Vulnerability[];
  truncated: boolean;
  license: string | null;
}

export interface SummaryTotals extends SeverityCounts {
  blocked: number;
  componentsScanned: number;
  componentsVulnerable: number;
  truncated: boolean;
}

export interface SummaryRepository extends SummaryTotals {
  repository: string;
  format: string;
}

export interface SecuritySummary {
  generatedAt: string;
  totals: SummaryTotals;
  repositories: SummaryRepository[];
}

export function severityOf(text: string): Severity {
  return (SEVERITIES as readonly string[]).includes(text) ? (text as Severity) : 'unknown';
}

function rankOf(severity: Severity): number {
  return SEVERITIES.length - SEVERITIES.indexOf(severity);
}

export function worstSeverity(values: readonly string[]): Severity | null {
  let worst: Severity | null = null;
  for (const value of values) {
    const severity = severityOf(value);
    if (worst === null || rankOf(severity) > rankOf(worst)) {
      worst = severity;
    }
  }
  return worst;
}

export function isUnavailable(error: unknown): boolean {
  return error instanceof ApiError && [401, 403, 404].includes(error.status);
}
