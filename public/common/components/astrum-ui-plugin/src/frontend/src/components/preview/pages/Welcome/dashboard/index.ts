

/**
 * Dashboard Components - REAL DATA ONLY
 * 
 * No MOCK data. All panels display data from real APIs.
 */

// Types
export type {
  Repository,
  RepositoryFormatSummary,
  InstanceTotals,
  SimplifiedPanelProps,
  RepositoriesByFormatPanelProps,
  InstanceTotalsPanelProps,
} from './simplified.types';

export type { DataPoint } from '../../../shared/Sparkline/Sparkline';

// Components
export { RepositoriesByFormatPanel } from './RepositoriesByFormatPanel';
export { InstanceTotalsPanel, type InstanceTotalsPanelWithSparklineProps } from './InstanceTotalsPanel';
export { QuickActionStatsPanel, type QuickActionStatsPanelProps } from './QuickActionStatsPanel';
export { Sparkline, type SparklineProps } from '../../../shared/Sparkline/Sparkline';

// Hooks
export { useRepositoriesByFormat } from './useRepositoriesByFormat';
export { useInstanceTotals } from './useInstanceTotals';
export { useMonthlyMetrics, formatBytesToGB } from './useMonthlyMetrics';
export { useInstanceStorage } from './useInstanceStorage';