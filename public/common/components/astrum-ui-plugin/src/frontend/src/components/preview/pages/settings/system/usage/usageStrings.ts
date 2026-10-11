

import localize from '../../../../../../interface/localize';

export const USAGE_LEARN_MORE_URL = 'http://links.globalart.dev/products/astrum/license/historical-usage';

export const USAGE_STRINGS = localize('react-ui', {
  TITLE: 'Usage',
  MENU_DESCRIPTION: 'Monitor historical usage trends.',
  HISTORICAL_TITLE: 'Historical Usage',
  DESCRIPTION: 'Monitor your storage usage trends over time.',
  LEARN_MORE: 'Learn how usage is calculated',
  UPDATE_FREQUENCY:
    'Storage usage metrics may take up to 72 hours to update. Recent repository activity, such as publishing, downloading, or deleting components, may not appear immediately.',
  STORAGE_NOTE_LABEL: 'Note:',
  STORAGE_NOTE:
    'This value may differ from the sum of individual repository storage totals. The storage usage includes: version history retained for 45 days after deletion, overwritten file versions retained for 30 days, and tenant access logs from the past 90 days.',
  LOAD_ERROR: 'Unable to load historical usage data. Please try again later.',
  PERMISSION_ERROR:
    'Insufficient Permissions: You need administrator privileges to view usage data.',
  EMPTY: 'No historical usage data available',
  RETRY: 'Retry',
  LOADING: 'Loading…',
  CHART_TITLE: 'Usage Insights',
  CHART_MONTH_LABEL: 'Month',
  CHART_LEGEND_EGRESS: 'Total Egress',
  CHART_LEGEND_STORAGE: 'Peak Storage',
  columns: {
    month: 'Month',
    totalEgress: 'Total Egress',
    egressChange: 'Egress % Change',
    peakStorage: 'Peak Storage',
    storageChange: 'Storage % Change',
    totalUsage: 'Total Usage',
  },
  tooltips: {
    egressChange: 'Change rate of the total monthly egress from the previous month.',
    storageChange: 'Change rate of the total monthly storage from the previous month.',
    totalUsage: 'Combined total of egress and storage usage. This is your total usage value.',
  },
}, 'PREVIEW.USAGE');
