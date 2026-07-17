

import { assign, send } from 'xstate';
import { mergeDeepRight } from 'ramda';
import { ListMachineUtils, ExtAPIUtils, APIConstants } from '@globalart/astrum-ui-plugin';

const {
  EXT: {
    PRIVILEGE: {
      ACTION,
      METHODS: { READ },
    },
    SMALL_PAGE_SIZE,
  },
} = APIConstants;

export default ListMachineUtils.buildListMachine({
  id: 'PrivilegesListMachine',
  sortableFields: ['name', 'description', 'type', 'permission'],
  apiSorting: true,
  apiFiltering: true,
  config: config =>
    mergeDeepRight(config, {
      context: {
        currentPage: 0,
        pageSize: SMALL_PAGE_SIZE,
        totalCount: 0,
      },
      states: {
        loading: {
          on: {
            FILTER: {
              target: 'loading',
              actions: ['setFilter', 'resetPage', 'cancelApiFilter', 'debounceApiFilter'],
            },
            CHANGE_PAGE: {
              target: 'loading',
              actions: ['setCurrentPage'],
            },
          },
        },
        loaded: {
          on: {
            CHANGE_PAGE: {
              target: 'loading',
              actions: ['setCurrentPage'],
            },
            FILTER: {
              target: 'loaded',
              actions: ['setFilter', 'resetPage', 'debounceApiFilter'],
            },
            API_FILTER: {
              target: 'loading',
            },
            CANCEL_API_FILTER: {
              target: 'loaded',
            },
          },
        },
      },
    }),
}).withConfig({
  actions: {
    setCurrentPage: assign({
      currentPage: (_, { page }) => page,
    }),
    setFilter: assign({
      filter: (_, { filter }) => filter,
    }),
    resetPage: assign({
      currentPage: 0,
    }),
    cancelApiFilter: send({ type: 'CANCEL_API_FILTER' }, { id: 'debounced-filter' }),
    debounceApiFilter: send({ type: 'API_FILTER' }, { delay: 400, id: 'debounced-filter' }),
    setData: assign({
      data: (_, event) => event.data?.data || [],
      totalCount: (_, event) => event.data?.total || 0,
    }),
  },
  services: {
    fetchData: ({ sortField, sortDirection, filter, currentPage, pageSize }) => {
      const params = {
        limit: pageSize,
        start: currentPage * pageSize,
        sortField,
        sortDirection,
        filterValue: filter,
      };
      return ExtAPIUtils.extAPIRequest(ACTION, READ.NAME, params).then(v => {
        return v.data.result;
      });
    },
  },
});
