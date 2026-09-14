
import { useCallback, useMemo, useState } from 'react';
import { useMachine } from '@xstate/react';
import { createUsageMachine } from './usageMachine';
import { createUsageChartMachine } from './usageChartMachine';
import { MonthOption, UseUsageReturn } from './types';

/**
 * Integration hook for the Cloud Usage module. Owns the monthly-metrics machine
 * (historical table) and the daily egress/storage chart machine (Usage Insights),
 * and tracks whether the dismissible storage-calculation note is still visible.
 * Both machines load on mount, mirroring the Classic Cloud Usage screen.
 */
export function useUsage(): UseUsageReturn {
  // useMemo([]) keeps each machine stable across re-renders within a mount.
  // On unmount, @xstate/react stops the services; a fresh mount creates new
  // instances and re-fetches. That re-fetch on remount is accepted XState v4
  // behaviour (sends to a stopped service are no-ops) — a useRef would not
  // change it, since refs also reset per mount.
  const usageMachine = useMemo(() => createUsageMachine(), []);
  const [state, send] = useMachine(usageMachine);

  const chartMachine = useMemo(() => createUsageChartMachine(), []);
  const [chartState, chartSend] = useMachine(chartMachine);

  const [storageNoteVisible, setStorageNoteVisible] = useState(true);

  const retry = useCallback(() => send({ type: 'RETRY' }), [send]);
  const retryChart = useCallback(() => chartSend({ type: 'RETRY' }), [chartSend]);
  const selectMonth = useCallback(
    (month: MonthOption) => chartSend({ type: 'SELECT_MONTH', month }),
    [chartSend],
  );
  const dismissStorageNote = useCallback(() => setStorageNoteVisible(false), []);

  return {
    loading: state.matches('loading'),
    error: state.context.loadError,
    isPermissionError: state.context.isPermissionError,
    metrics: state.context.metrics,
    retry,
    storageNoteVisible,
    dismissStorageNote,
    chartLoading: chartState.matches('loading'),
    chartError: chartState.context.loadError,
    chartData: chartState.context.combinedData,
    monthOptions: chartState.context.monthOptions,
    selectedMonth: chartState.context.selectedMonth,
    selectMonth,
    retryChart,
  };
}
