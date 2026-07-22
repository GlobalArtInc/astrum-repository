
import {assign} from 'xstate';
import {mergeDeepRight, indexBy, prop} from 'ramda';

import {ListMachineUtils, ExtAPIUtils, APIConstants} from '@globalart/astrum-ui-plugin';

const {EXT: {USER: {ACTION, METHODS: {READ, READ_SOURCES}}, MIDDLE_PAGE_SIZE}} = APIConstants;

export default ListMachineUtils.buildListMachine({
  id: 'UsersListMachine',
  initial: 'loadingSources',
  sortableFields: ['userId', 'realm', 'firstName', 'lastName', 'email', 'status'],
  sortField: 'userId',
  apiFiltering: true,
  config: (config) =>
      mergeDeepRight(config, {
        context: {
          sourceFilter: 'default',
        },
        states: {
          loadingSources: {
            invoke: {
              id: 'fetchSources',
              src: 'fetchSources',
              onDone: {
                target: 'loading',
                actions: ['setSources']
              },
              onError: {
                target: 'error',
                actions: ['setError']
              }
            }
          },
          loaded: {
            on: {
              FILTER_BY_SOURCE: {
                target: 'loaded',
                actions: ['setSourceFilter', 'debounceApiFilter', 'apiFilter'],
              },
            },
          },
        },
      })
}).withConfig({
  actions: {
    setData: assign({
      data: (_, event) => event.data?.data?.result?.data,
      pristineData: (_, event) => event.data?.data?.result?.data,
    }),
    setSources: assign({
      sources: (_, event) => indexBy(prop('id'), event.data?.data?.result?.data || []),
    }),
    setSourceFilter: assign({
      sourceFilter: (_, {filter}) => filter
    }),
  },
  services: {
    fetchSources: () => ExtAPIUtils.extAPIRequest(ACTION, READ_SOURCES),
    fetchData: ({filter, sourceFilter}) => {
      return ExtAPIUtils.extAPIRequest(ACTION, READ, {
        limit: MIDDLE_PAGE_SIZE,
        filter: [
          {property: 'userId', value: filter},
          {property: 'source', value: sourceFilter},
        ],
      });
    },
  }
});
