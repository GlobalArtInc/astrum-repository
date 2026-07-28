
import {assign, createMachine} from 'xstate';
import Axios from 'axios';
import {KEY_STORAGE, KEY_EGRESS, getMonthOptions, getDateRange, isPermissionError} from "./UsageInsightsUtils";

/**
 * State machine for fetching daily metrics (egress and storage) from DailyMetricsApiResource
 *
 * Context is initialized in the entry action of the loading state
 */
export default createMachine(
    {
      id: 'UsageInsightsChartMachine',
      initial: 'loading',

      context: {
        egressData: null,
        storageData: null,
        combinedData: null,
        loadError: null,
        isPermissionError: false,
        monthOptions: [],
        selectedMonth: null,
        dateFrom: '',
        dateTo: '',
        isOpen: false
      },

      states: {
        loading: {
          entry: 'ensureMonthSelection',
          invoke: {
            id: 'fetchMetrics',
            src: 'fetchAllMetrics',
            onDone: {
              target: 'loaded',
              actions: 'setAllData'
            },
            onError: {
              target: 'loadError',
              actions: 'setError'
            }
          }
        },
        loaded: {
          on: {
            REFRESH: {
              target: 'loading',
              actions: 'clearError'
            },
            SELECT_MONTH: {
              target: 'loading',
              actions: ['selectMonth', 'closeDropdown', 'clearData', 'clearError']
            },
            TOGGLE_DROPDOWN: {
              actions: 'toggleDropdown'
            }
          }
        },
        loadError: {
          on: {
            RETRY: {
              target: 'loading',
              actions: 'clearError'
            }
          }
        }
      }
    },
    {
      actions: {
        ensureMonthSelection: assign((context) => {
          // Initialize month options on first load
          if (context.monthOptions.length === 0) {
            const monthOptions = getMonthOptions();
            const selectedMonth = monthOptions.length > 0 ? monthOptions[0] : null;
            const dateRange = selectedMonth ? selectedMonth.value : getDateRange(new Date());

            return {
              monthOptions,
              selectedMonth,
              dateFrom: dateRange.dateFrom,
              dateTo: dateRange.dateTo
            };
          }
          return {};
        }),
        selectMonth: assign({
          selectedMonth: (_, event) => event.month,
          dateFrom: (_, event) => event.month?.value?.dateFrom || '',
          dateTo: (_, event) => event.month?.value?.dateTo || ''
        }),
        toggleDropdown: assign({
          isOpen: (context) => !context.isOpen
        }),
        closeDropdown: assign({
          isOpen: () => false
        }),
        clearData: assign({
          egressData: () => null,
          storageData: () => null,
          combinedData: () => null
        }),
        setAllData: assign((context, event) => {
          const egressData = event.data[0].data;
          const storageData = event.data[1].data;

          // Combine immediately
          const metricsMap = new Map();
          const appendToMetrics = (item, key) => {
            const date = item.date;
            const value = metricsMap.get(date) || {metricDate: date, [KEY_EGRESS]: 0, [KEY_STORAGE]: 0, _available: {}};
            value[key] = Number(item.bytes) || 0;
            value._available[key] = true;
            metricsMap.set(date, value);
          };

          egressData?.data?.forEach(item => appendToMetrics(item, KEY_EGRESS));
          storageData?.data?.forEach(item => appendToMetrics(item, KEY_STORAGE));

          const combinedData = Array.from(metricsMap.values()).sort(
              (a, b) => new Date(a.metricDate) - new Date(b.metricDate)
          );

          return { egressData, storageData, combinedData };
        }),
        setError: assign((_, event) => {
          const error = event.data;
          const url = error.config?.url || '';

          // Identify which endpoint failed
          const metric = url.includes('egress') ? 'egress'
              : url.includes('storage') ? 'storage'
                  : 'metrics';

          // Build user-friendly message
          const baseMessage = error.response?.data?.message
              || error.message
              || 'Unknown error';

          return {
            loadError: `Failed to load ${metric} data: ${baseMessage}`,
            isPermissionError: isPermissionError(error)
          };
        }),
        clearError: assign({
          loadError: () => null,
          isPermissionError: () => false
        })
      },
      services: {
        fetchAllMetrics: (context) => {
          const params = { dateFrom: context.dateFrom, dateTo: context.dateTo };
          return Promise.all([
            Axios.get('service/rest/v1/daily-metrics/egress', {params}),
            Axios.get('service/rest/v1/daily-metrics/storage', {params})
          ]);
        }
      }
    }
);
