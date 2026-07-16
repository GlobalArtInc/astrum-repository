
import Axios from 'axios';
import {assign} from 'xstate';

import {ExtJS, FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {ERROR, CROWD_SETTINGS} = UIStrings;

export default FormUtils.buildFormMachine({
  id: 'CrowdSettingsForm',
  stateAfterSave: 'loading',
  config: (config) => ({
    ...config,
    states: {
      ...config.states,
      loaded: {
        ...config.states.loaded,
        on: {
          ...config.states.loaded.on,
          'VERIFY-CONNECTION': 'verifyingConnection',
          'CLEAR-CACHE': 'clearingCache'
        }
      },
      verifyingConnection: {
        invoke: {
          id: 'verifyConnection',
          src: 'verifyConnection',
          onDone: {
            target: 'loaded',
            actions: ['logConnectionSuccess']
          },
          onError: {
            target: 'loaded',
            actions: ['logConnectionError']
          }
        }
      },
      clearingCache: {
        invoke: {
          id: 'clearCache',
          src: 'clearCache',
          onDone: {
            target: 'loading',
            actions: ['logClearCacheSuccess']
          },
          onError: {
            target: 'loaded',
            actions: ['logClearCacheError']
          }
        }
      }
    }
  })
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data}) => ({
        url: ValidationUtils.notUri(data.url) ? CROWD_SETTINGS.FIELDS.urlValidationError : null,
        applicationName: ValidationUtils.isBlank(data.applicationName) ? ERROR.FIELD_REQUIRED : null,
        applicationPassword: ValidationUtils.isBlank(data.applicationPassword) ? ERROR.FIELD_REQUIRED : null,
        timeout: ValidationUtils.isInRange({value: data.timeout, min: 1, max: 3600})
      })
    }),

    logLoadError: ({error}) => {
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(UIStrings.CROWD_SETTINGS.MESSAGES.LOAD_ERROR);
    },
    logSaveSuccess: () => {
      ExtJS.showSuccessMessage(UIStrings.CROWD_SETTINGS.MESSAGES.SAVE_SUCCESS);
    },
    logSaveError: ({error}) => {
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(UIStrings.CROWD_SETTINGS.MESSAGES.SAVE_ERROR);
    },
    logConnectionError: (_, event) => {
      const raw = event.data?.response?.data;
      const detail = raw?.message ?? (typeof raw === 'string' ? raw : '');
      ExtJS.showErrorMessage(`${CROWD_SETTINGS.MESSAGES.VERIFY_CONNECTION_ERROR} ${detail}`);
    },
    logConnectionSuccess: () => {
      ExtJS.showSuccessMessage(CROWD_SETTINGS.MESSAGES.VERIFY_CONNECTION_SUCCESS);
    },
    logClearCacheError: (_, event) => {
      const raw = event.data?.response?.data;
      const detail = raw?.message ?? (typeof raw === 'string' ? raw : '');
      ExtJS.showErrorMessage(`${CROWD_SETTINGS.MESSAGES.CLEAR_CACHE_ERROR} ${detail}`);
    },
    logClearCacheSuccess: () => {
      ExtJS.showSuccessMessage(CROWD_SETTINGS.MESSAGES.CLEAR_CACHE_SUCCESS);
    }
  },

  guards: {
    canSave: ({isPristine, validationErrors}) => {
      const isValid = !FormUtils.isInvalid(validationErrors);
      return !isPristine && isValid;
    }
  },

  services: {
    fetchData: () => {
      return Axios.get('service/rest/v1/security/atlassian-crowd');
    },
    saveData: ({data}) => {
      return Axios.put('service/rest/v1/security/atlassian-crowd', {
        ...data
      });
    },
    verifyConnection: ({data}) => {
      return Axios.post('service/rest/v1/security/atlassian-crowd/verify-connection', {
        ...data
      });
    },
    clearCache: () => {
      return Axios.post('service/rest/v1/security/atlassian-crowd/clear-cache');
    }
  }
});

