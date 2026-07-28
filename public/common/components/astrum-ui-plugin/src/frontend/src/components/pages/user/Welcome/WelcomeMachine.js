
import {assign, createMachine} from 'xstate';

import { ExtAPIUtils, ExtJS, APIConstants } from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const action = APIConstants.EXT.OUTREACH.ACTION,
    outreachStatusMethod = APIConstants.EXT.OUTREACH.METHODS.READ_STATUS,
    proxyDownloadNumbersMethod = APIConstants.EXT.OUTREACH.METHODS.GET_PROXY_DOWNLOAD_NUMBERS;

const welcomeMachine = createMachine({
  id: 'WelcomeMachine',
  predictableActionArguments: true,

  initial: 'loaded',

  states: {
    loading: {
      invoke: {
        src: 'fetch',
        onDone: {
          target: 'loaded',
          actions: ['setData']
        },
        onError: {
          target: 'error',
          actions: ['setError']
        }
      }
    },
    loaded: {},
    error: {}
  },

  on: {
    LOAD: {
      target: 'loading'
    }
  }
}, {
  actions: {
    setData: assign({
      data: (_, {data}) => data
    }),

    setError: assign({
      error: (_, event) => event?.data?.message || UIStrings.ERROR.UNKNOWN
    })
  },
  services: {
    fetch: async () => {
      const CORE_EDITION = 'CORE',
          COMMUNITY_EDITION = 'COMMUNITY',
          user = ExtJS.state().getUser(),
          edition = ExtJS.state().getValue('status')?.edition;

      const getOutreachData = async () => {

        if (edition === CORE_EDITION) {
          return {
            proxyDownloadNumberParams: {}
          };
        }

        const requiresOutreach = edition === COMMUNITY_EDITION || !!user,
            outreachStatusRequest = { action, method: outreachStatusMethod },
            proxyDownloadNumbersRequest = { action, method: proxyDownloadNumbersMethod },
            requests = [outreachStatusRequest, proxyDownloadNumbersRequest],

            bulkResponse = await ExtAPIUtils.extAPIBulkRequest(requests),
            outreachStatusResponse = bulkResponse.data.find(({ method }) => method === outreachStatusRequest.method),
            proxyDownloadNumbersResponse = bulkResponse.data
                .find(({ method }) => method === proxyDownloadNumbersRequest.method),

            // The ExtAPIUtils expect this extra layer of object
            wrappedOutreachStatusResponse = { data: outreachStatusResponse },
            wrappedProxyDownloadNumbersResponse = { data: proxyDownloadNumbersResponse };

        ExtAPIUtils.checkForError(wrappedOutreachStatusResponse);

        // the outreach response includes a `data` property that is a long hexadecimal string (when the iframe should
        // be enabled) or null when the iframe should be disabled
        const showOutreachIframe = requiresOutreach &&
                Boolean(outreachStatusResponse?.result?.success) && outreachStatusResponse?.result?.data !== null,
            proxyDownloadNumberParams = ExtAPIUtils.extractResult(wrappedProxyDownloadNumbersResponse);

        return {
          showOutreachIframe,
          proxyDownloadNumberParams,
        };
      }

      return await getOutreachData();

    }
  }
});

export default welcomeMachine;
