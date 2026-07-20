
import Axios from 'axios';
import {assign, Machine} from 'xstate';
import {mergeDeepRight} from 'ramda';
import {APIConstants} from '@globalart/astrum-ui-plugin';

export default Machine(
  {
    id: 'NodeCardMachine',
    initial: 'loading',
    context: {
      data: null,
      pristineData: null,
    },
    states: {
      loading: {
        invoke: {
          src: 'fetchData',
          onDone: {
            target: 'loaded',
            actions: 'setData',
          },
        },
      },
      loaded: {
        on: {
          UPDATE_STATUS: {
            target: 'loading',
            actions: 'updateStatus',
          },
          SET_STATUS_LOCAL: {
            actions: 'updateStatus',
          },
        },
        after: {
          TIMEOUT: {
            target: 'loading',
            cond: 'canCheckStatus',
          },
        },
      },
    },
  },
  {
    actions: {
      setData: assign({
        data: (context, event) => event?.data?.data || context?.data,
      }),
      updateStatus: assign({
        data: (context, {status}) => mergeDeepRight(context.data, {status}),
      }),
    },

    guards: {
      canCheckStatus: ({data}) => data.status === 'CREATING',
    },

    services: {
      fetchData: (context) =>
        Axios.get(
          APIConstants.REST.INTERNAL.GET_ZIP_STATUS + context.data?.nodeId
        ),
    },

    delays: {
      TIMEOUT: 2000,
    },
  }
);
