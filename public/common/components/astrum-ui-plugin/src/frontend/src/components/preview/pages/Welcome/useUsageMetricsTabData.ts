
import { ExtJS } from '../../../../interface/ExtJS';
// monthlyMetrics is only needed for the cloud path (CloudUsageCenterPanel).
// Self-hosted path uses UsageCenter which reads its own state internally (ASTRUM-53863).
import { useMonthlyMetrics } from './dashboard';

export function useUsageMetricsTabData() {
  const isCloud: boolean = ExtJS.state().getValue('isCloud', false);
  const monthlyMetrics = useMonthlyMetrics();

  return {isCloud, monthlyMetrics};
}
