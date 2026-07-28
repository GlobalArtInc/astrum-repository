
import { assign, createMachine } from 'xstate';

import Axios from 'axios';
import { isPermissionError } from './UsageInsightsUtils';

export default createMachine(
  {
    id: 'LicensingHistoricalUsageMachine',
    initial: 'loading',

    context: {
      data: null,
      pristineData: null,
      loadError: null,
      isPermissionError: false
    },

    states: {
      loading: {
        invoke: {
          src: 'fetchData',
          onDone: {
            target: 'loaded',
            actions: ['setData', 'clearError']
          },
          onError: {
            target: 'loadError',
            actions: assign({
              loadError: (context, event) => event.data,
              isPermissionError: (context, event) => isPermissionError(event.data)
            })
          }
        }
      },
      loaded: {},
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
      setData: assign({
        data: (_, { data }) => data.data,
        pristineData: (_, { data }) => data.data
      }),
      clearError: assign({
        loadError: () => null,
        isPermissionError: () => false
      })
    },
    services: {
      fetchData: () => Axios.get('service/rest/v1/monthly-metrics')
    }
  }
);
