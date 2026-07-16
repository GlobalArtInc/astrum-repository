
import Axios from 'axios';
import {assign} from 'xstate';

import {ExtJS, FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {DATASTORE_CONFIGURATION} = UIStrings;

export default FormUtils.buildFormMachine({
  id: 'DataStoreConfigurationForm'
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data}) => ({
        maximumConnectionPool: ValidationUtils.isInRange({
          value: data.maximumConnectionPool,
          min: 1,
          max: 3000,
          allowDecimals: false,
        })
      })
    }),
    setData: assign({
      data: (_, {data}) => data.data,
      pristineData: (_, {data}) => data.data
    }),

    logSaveError: (_, event) => {
      const error = event.data.data;
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(DATASTORE_CONFIGURATION.MESSAGES.SAVE_ERROR);
    },
    logSaveSuccess: () => ExtJS.showSuccessMessage(DATASTORE_CONFIGURATION.MESSAGES.SAVE_SUCCESS),

    logLoadError: (_, event) => {
      const error = event.data.data;
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(DATASTORE_CONFIGURATION.MESSAGES.LOAD_ERROR);
    }
  },
  services:
      {
        fetchData: async () => {
          await ExtJS.waitForPermissions();
          return Axios.get('service/rest/internal/ui/datastore');
        },
        saveData: ({data}) => {
          return Axios.put('service/rest/internal/ui/datastore', data);
        }
      }
});

