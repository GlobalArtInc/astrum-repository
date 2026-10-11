import { restClient, API_INTERNAL_UI } from '../../../../../interface/api';
import type { UsageDashboardData } from './usageDashboard.types';

export const USAGE_DASHBOARD_URL = `${API_INTERNAL_UI}/usage-dashboard`;

/**
 * Reads the usage of the repositories the user may browse over the last `days` days, of one repository when one is
 * named.
 */
export function fetchUsageDashboard(
  days: number,
  repository: string | null,
  signal?: AbortSignal
): Promise<UsageDashboardData> {
  const params: Record<string, string | number> = { days };
  if (repository) {
    params.repository = repository;
  }
  return restClient.get<UsageDashboardData>(USAGE_DASHBOARD_URL, { params, signal });
}
