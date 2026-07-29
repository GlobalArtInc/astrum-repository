
import {ExtJS} from '../../../../interface/ExtJS';
import {
  OVER_LIMITS,
  NEAR_LIMITS,
  UNDER_LIMITS,
  CE_REQUESTS_HARD_THRESHOLD,
  CE_COMPONENTS_HARD_THRESHOLD,
  STORAGE_KEY_CE_THROTTLING_STATUS,
  STORAGE_KEY_CE_GRACE_PERIOD_ENDS,
  isTestSessionActive,
  getMetricData,
  resolveThrottlingStatus,
} from './usageHelperUtils';

// Re-export the shared constants so existing consumers of this module keep working.
export {OVER_LIMITS, NEAR_LIMITS, UNDER_LIMITS, CE_REQUESTS_HARD_THRESHOLD, CE_COMPONENTS_HARD_THRESHOLD};

// ExtJS-reading helpers stay per-plugin (usageHelperUtils.js is ExtJS-free);
// duplicated with the other plugin — keep in lockstep (ASTRUM-54019).

function addProductParams() {
  const nodeId = ExtJS.state().getValue('astrum.node.id');
  const usage = ExtJS.state().getValue('contentUsageEvaluationResult', []);
  const { metricValue: peakRequestsMetricValue, thresholdValue: peakRequestsThresholdValue, highestRecordedCount: highestRecordedCountPeakRequests } = getMetricData(usage, "peak_requests_per_day");
  const { metricValue: componentTotalMetricValue, thresholdValue: componentTotalThresholdValue, highestRecordedCount: highestRecordedCountComponentTotal } = getMetricData(usage, "component_total_count");
  const malwareCount = ExtJS.state().getValue('astrum.malware.count')?.totalCount || 0;

  const params =
      {
        nodeId: nodeId,
        componentCountLimit: componentTotalThresholdValue,
        componentCountMax: highestRecordedCountComponentTotal,
        componentCount: componentTotalMetricValue,
        requestsPer24HoursLimit: peakRequestsThresholdValue,
        requestsPer24HoursMax: highestRecordedCountPeakRequests,
        requestsPer24HoursCount: peakRequestsMetricValue,
        malwareCount: malwareCount
      };
  return new URLSearchParams(params).toString();
}

function useViewPurchaseALicenseUrl() {
  return `http://links.globalart.dev/products/astrum/ce/purchase-license?${addProductParams()}`;
}

function buildLearnMoreUrl(throttlingStatus) {
  if (throttlingStatus === 'OVER_LIMITS_GRACE_PERIOD_ENDED') {
    return `http://links.globalart.dev/products/astrum/ce/learn-more-limits-enforced?${addProductParams()}`;
  }
  return `http://links.globalart.dev/products/astrum/ce/learn-more?${addProductParams()}`;
}

function useViewLearnMoreUrl() {
  return buildLearnMoreUrl(useThrottlingStatus());
}

function useGracePeriodEndsDate() {
  const testGracePeriod = isTestSessionActive() && typeof localStorage !== 'undefined' && localStorage.getItem(STORAGE_KEY_CE_GRACE_PERIOD_ENDS);
  if (testGracePeriod) {
    return new Date(testGracePeriod);
  }
  return new Date(ExtJS.state().getValue('astrum.community.gracePeriodEnds'));
}

function useThrottlingStatusValue () {
  const testOverride = isTestSessionActive() && typeof localStorage !== 'undefined' && localStorage.getItem(STORAGE_KEY_CE_THROTTLING_STATUS);
  if (testOverride) {
    return testOverride;
  }
  return ExtJS.state().getValue('astrum.community.throttlingStatus');
}

function useGracePeriodEndDate() {
  const gracePeriodEnds = ExtJS.useState(useGracePeriodEndsDate);
  return gracePeriodEnds.toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' });
}

function useDaysUntilGracePeriodEnds() {
  const gracePeriodEnds = ExtJS.useState(useGracePeriodEndsDate);
  const now = new Date();

  const diffInMs = gracePeriodEnds ?
    Date.UTC(gracePeriodEnds.getFullYear(), gracePeriodEnds.getMonth(), gracePeriodEnds.getDate()) -
    Date.UTC(now.getFullYear(), now.getMonth(), now.getDate()) : 0;
  const diffInDays = Math.round(diffInMs / (1000 * 60 * 60 * 24));
  return diffInDays;
}

function useThrottlingStatus() {
  const throttlingStatus = ExtJS.useState(useThrottlingStatusValue);
  const diffInDays = useDaysUntilGracePeriodEnds();
  const isAdmin = ExtJS.useUser()?.administrator;
  return resolveThrottlingStatus(throttlingStatus, diffInDays, isAdmin);
}

export const helperFunctions = {
  useViewLearnMoreUrl,
  buildLearnMoreUrl,
  useViewPurchaseALicenseUrl,
  useGracePeriodEndDate,
  useThrottlingStatus,
  useGracePeriodEndsDate,
  useThrottlingStatusValue,
  getMetricData,
  OVER_LIMITS,
  NEAR_LIMITS,
  UNDER_LIMITS,
  useDaysUntilGracePeriodEnds,
};
