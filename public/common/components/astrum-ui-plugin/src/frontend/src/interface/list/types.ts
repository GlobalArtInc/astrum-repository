

/**
 * Sort direction for list items
 */
export type SortDirection = 'asc' | 'desc' | null;

/**
 * Base context for list machine
 */
export interface ListMachineContext<T = unknown, TFilters = Record<string, unknown>> {
  /** Raw data loaded from API */
  pristineData: T[];
  /** Filtered and sorted data for display */
  data: T[];
  /** Current sort field */
  sortField: string;
  /** Current sort direction */
  sortDirection: SortDirection;
  /** All filters (text search and structured filters) */
  filters: TFilters;
  /** Error message if load failed */
  error: string | null;
}

/**
 * Events for list machine
 */
export type ListMachineEvent<TFilters = Record<string, unknown>> =
  | { type: 'LOAD' }
  | { type: 'SET_FILTERS'; filters: Partial<TFilters> }
  | { type: 'CLEAR_FILTERS' }
  | { type: 'SORT'; field: string }
  | { type: 'SET_DATA'; data: unknown[] };

/**
 * Initial configuration for creating a list machine (builder pattern)
 */
export interface ListMachineConfig<_T = unknown, TFilters = Record<string, unknown>> {
  /** Unique machine ID */
  id: string;
  /** Initial context values */
  context?: {
    /** Initial sort field */
    sortField?: string;
    /** Initial sort direction */
    sortDirection?: SortDirection;
    /** Initial filters */
    filters?: TFilters;
  };
}

/**
 * Configuration for withConfig() method
 */
export interface ListMachineWithConfig<T = unknown, TFilters = Record<string, unknown>> {
  /** Services for the machine */
  services?: {
    /** Service to fetch data */
    fetchData?: () => Promise<T[]>;
  };
  /** Actions for the machine */
  actions?: {
    /** Custom filter function (filters pristineData based on filters) */
    filterData?: (context: ListMachineContext<T, TFilters>) => T[];
    /** Custom sort function (sorts filtered data) */
    sortData?: (context: ListMachineContext<T, TFilters>) => T[];
  };
}
