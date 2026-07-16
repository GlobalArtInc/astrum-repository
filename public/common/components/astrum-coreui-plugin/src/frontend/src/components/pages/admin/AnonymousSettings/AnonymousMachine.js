
import {assign} from 'xstate';
import Axios from 'axios';
import {ExtJS, FormUtils, ValidationUtils, APIConstants} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {ERROR, ANONYMOUS_SETTINGS} = UIStrings;

const {
  REALMS_TYPES,
  ANONYMOUS_SETTINGS: ANONYMOUS_API
} = APIConstants.REST.INTERNAL;

export default FormUtils.buildFormMachine({
  id: 'AnonymousSettingsForm'
}).withConfig({
  actions: {
    setData: assign({
      data: (_, {data: [, settings]}) => settings.data,
      pristineData: (_, {data: [, settings]}) => settings.data,
      realms: (_, {data: [realms]}) => realms.data
    }),

    validate: assign({
      validationErrors: ({data}) => ({
        userId: ValidationUtils.isBlank(data?.userId) ? ERROR.FIELD_REQUIRED : null
      })
    }),

    logLoadError: (_, {error}) => {
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(ANONYMOUS_SETTINGS.MESSAGES.LOAD_ERROR);
    },

    logSaveError: (_, {error}) => {
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(ANONYMOUS_SETTINGS.MESSAGES.SAVE_ERROR);
    },
    logSaveSuccess: () => ExtJS.showSuccessMessage(ANONYMOUS_SETTINGS.MESSAGES.SAVE_SUCCESS)
  },
  services: {
    fetchData: async () => {
      await ExtJS.waitForPermissions();
      return Axios.all([
        Axios.get(REALMS_TYPES),
        Axios.get(ANONYMOUS_API)
      ]);
    },
    saveData: ({data}) => Axios.put(ANONYMOUS_API, {...data, userId: data.userId.trim()})
  }
});
