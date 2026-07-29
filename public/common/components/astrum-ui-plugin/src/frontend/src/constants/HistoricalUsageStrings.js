
import {NxTextLink} from "@globalart/react-shared-components";
import React from "react";
import {faServer} from "@fortawesome/free-solid-svg-icons";

export default {
  HISTORICAL_USAGE: {
    MENU: 'Usage',
    MENU_DESCRIPTION: 'Monitor historical usage trends.',
    TITLE: 'Historical Usage',
    USAGE_DATA_UPDATE_FREQUENCY: 'Storage usage metrics may take up to 72 hours to update. Recent repository activity, such as publishing, downloading, or deleting components, may not appear immediately.',
    USAGE_DATA_STORAGE_EXPLANATION: <>
      <strong>Note:</strong> This value may differ from the sum of individual repository storage totals. The storage usage includes: version history retained for 45 days after deletion, overwritten file versions retained for 30 days, and tenant access logs from the past 90 days.
    </>,
    DESCRIPTION: <>Monitor your storage usage trends over time. <NxTextLink external
                                                                               href="http://links.globalart.dev/products/astrum/license/historical-usage">Learn
      how usage is calculated</NxTextLink></>,
    PERMISSION_ERROR: 'Insufficient Permissions: You need administrator privileges to view usage data.',
    LOAD_ERROR: 'Unable to load historical usage data. Please try again later.',
    MONTH: 'Month',
    PEAK_COMPONENTS: 'Peak Components',
    COMPONENTS_CHANGE: 'Components % Change',
    COMPONENTS_CHANGE_TOOLTIP: 'Change rate of the peak component count from the previous month.',
    TOTAL_REQUESTS: 'Total Requests',
    REQUESTS_CHANGE: 'Requests % Change',
    REQUESTS_CHANGE_TOOLTIP: 'Change rate of the total monthly requests from the previous month.',
    PEAK_STORAGE: 'Peak Storage',
    STORAGE_CHANGE: '% Change Storage',
    STORAGE_CHANGE_TOOLTIP: 'Change rate of the total monthly storage from the previous month.',
    TOTAL_EGRESS: 'Total Egress',
    TOTAL_EGRESS_TOOLTIP: 'Egress is based on application-level tracking and may differ from actual network transfer measured by your cloud provider.',
    EGRESS_CHANGE: 'Egress % Change',
    EGRESS_CHANGE_TOOLTIP: 'Change rate of the total monthly egress from the previous month.',
    TOTAL_USAGE: 'Total Usage',
    TOTAL_USAGE_TOOLTIP: 'Combined total of egress and storage usage. This is your total usage value.',
    ICON: faServer,
    CHART: {
      LEGEND_EGRESS: 'Total Egress',
      LEGEND_STORAGE: 'Peak Storage',
      FILTER_OPTION_HEADER: 'Month',
      AXIS_DAYS: 'Days',
      AXIS_EGRESS_STORAGE: "Egress and Storage",
      TITLE: "Usage Insights",
      SUB_TITLE: "Month",
      DATA_NOT_AVAILABLE: 'Data not available',
      PERMISSION_ERROR: 'Insufficient Permissions: You need administrator privileges to view usage insights.'
    }
  }
};
