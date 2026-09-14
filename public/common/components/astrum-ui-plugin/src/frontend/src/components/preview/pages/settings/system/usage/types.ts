

/**
 * One row of the monthly-metrics REST response for the Cloud Usage screen.
 * The Cloud backend returns `egress` and `storage` (bytes) plus their
 * month-over-month percentage changes. `totalUsage` is derived (egress + storage).
 */
export interface MonthlyMetric {
  metricDate: string;
  egress: number | string | null;
  storage: number | string | null;
  percentageChangeEgress: number | string | null;
  percentageChangeStorage: number | string | null;
}

/** A single day's combined egress + storage for the chart. */
export interface ChartDataPoint {
  metricDate: string;
  egress: number;
  storage: number;
}

/** A selectable month for the chart dropdown. */
export interface MonthOption {
  key: string;
  label: string;
  value: { dateFrom: string; dateTo: string };
}

/** Contract exposed by the useUsage hook to the view layer. */
export interface UseUsageReturn {
  loading: boolean;
  error: string | null;
  isPermissionError: boolean;
  metrics: MonthlyMetric[];
  retry: () => void;
  // Dismissible storage-calculation note
  storageNoteVisible: boolean;
  dismissStorageNote: () => void;
  // Usage Insights chart (daily egress/storage)
  chartLoading: boolean;
  chartError: string | null;
  chartData: ChartDataPoint[];
  monthOptions: MonthOption[];
  selectedMonth: MonthOption | null;
  selectMonth: (month: MonthOption) => void;
  retryChart: () => void;
}
