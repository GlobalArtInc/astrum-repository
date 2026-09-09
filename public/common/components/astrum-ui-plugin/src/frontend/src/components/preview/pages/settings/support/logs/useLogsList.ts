

import { useCallback, useMemo } from 'react';
import { useMachine } from '@xstate/react';
import { logsListMachine } from './LogsListMachine';
import { LogFile, LogSortField, SortDirection } from './types';

export interface UseLogsListReturn {
  filteredLogs: LogFile[];
  filter: string;
  sortField: LogSortField;
  sortDirection: SortDirection;
  error: string | null;
  isLoading: boolean;
  setFilter: (value: string) => void;
  handleSort: (field: LogSortField) => void;
}

export function useLogsList(): UseLogsListReturn {
  const [state, send] = useMachine(logsListMachine);
  const { logs, filter, sortField, sortDirection, error } = state.context;

  const isLoading = state.matches('loading');

  const filteredLogs = useMemo(() => {
    let result = [...logs];

    if (filter) {
      const lowerFilter = filter.toLowerCase();
      result = result.filter((log) =>
        log.fileName.toLowerCase().includes(lowerFilter)
      );
    }

    result.sort((a, b) => {
      let comparison = 0;
      switch (sortField) {
        case 'fileName':
          comparison = a.fileName.localeCompare(b.fileName);
          break;
        case 'size':
          comparison = a.size - b.size;
          break;
        case 'lastModified':
          comparison = a.lastModified - b.lastModified;
          break;
      }
      return sortDirection === 'asc' ? comparison : -comparison;
    });

    return result;
  }, [logs, filter, sortField, sortDirection]);

  const setFilter = useCallback((value: string) => {
    send({ type: 'SET_FILTER', value });
  }, [send]);

  const handleSort = useCallback((field: LogSortField) => {
    send({ type: 'SORT', field });
  }, [send]);

  return {
    filteredLogs,
    filter,
    sortField,
    sortDirection,
    error,
    isLoading,
    setFilter,
    handleSort,
  };
}
