
import {assign} from 'xstate';
import Axios from 'axios';
import {mergeDeepRight} from 'ramda';

import {
  FormUtils,
  APIConstants,
  ExtAPIUtils,
  ValidationUtils
} from '@globalart/astrum-ui-plugin';

import {URLS, remoteHostRequestData} from './SslCertificatesHelper';

const {EXT} = APIConstants;

export const SOURCES = {
  REMOTE_HOST: 'remoteHost',
  PEM: 'PEM'
};

const EMPTY_DATA = {
  remoteHostUrl: '',
  pemContent: ''
};

export default FormUtils.buildFormMachine({
  id: 'SslCertificatesAddFormMachine',
  initial: 'loaded',
  config: (config) =>
      mergeDeepRight(config, {
        context: {
          pristineData: EMPTY_DATA,
          data: EMPTY_DATA,
          source: SOURCES.REMOTE_HOST,
        },
        states: {
          loaded: {
            on: {
              SET_SOURCE: {
                actions: 'setSource',
                target: 'loaded',
              },
              LOAD_DETAILS: {
                target: 'loadingDetails',
              },
            }
          },
          loadingDetails: {
            entry: 'clearSaveError',
            invoke: {
              src: 'fetchDetails',
              onDone: {
                target: 'previewDetails',
                actions: ['clearSaveError', 'setLoadDetailsData'],
              },
              onError: {
                target: 'loaded',
                actions: ['setSaveError', 'logSaveError']
              }
            }
          },
          previewDetails: {
            on: {
              ADD_CERTIFICATE: {
                target: 'saving',
                cond: 'canSave'
              },
            },
          },
          saving: {
            invoke: {
              onError: {
                target: 'previewDetails',
              },
            },
          },
        }
      })
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data, source}) => ({
        remoteHostUrl: source === SOURCES.REMOTE_HOST ? ValidationUtils.validateNotBlank(data.remoteHostUrl): null,
        pemContent: source === SOURCES.PEM ? ValidationUtils.validateNotBlank(data.pemContent) : null,
      })
    }),
    setLoadDetailsData: assign({
      data: (_, event) => event.data,
    }),
    setSource: assign(({data}, {value}) => {
      let {remoteHostUrl, pemContent} = data;
      return {
        source: value,
        data: {
          ...data,
          remoteHostUrl: value === SOURCES.REMOTE_HOST ? remoteHostUrl : '',
          pemContent: value === SOURCES.PEM ? pemContent : ''
        }
      };
    }),
  },
  services: {
    fetchDetails: async ({data: {remoteHostUrl, pemContent}}) => {
      let response;
      if (remoteHostUrl) {
        response = await ExtAPIUtils.extAPIRequest(EXT.SSL.ACTION, EXT.SSL.METHODS.RETRIEVE_FROM_HOST, {
          data: remoteHostRequestData(remoteHostUrl)
        });
      } else if (pemContent) {
        response = await ExtAPIUtils.extAPIRequest(EXT.SSL.ACTION, EXT.SSL.METHODS.DETAILS, {
          data: [pemContent]
        });
      } else {
        return {};
      }

      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    },
    saveData: ({data}) => Axios.post(URLS.createSslCertificatesUrl, data?.pem),
  }
});
