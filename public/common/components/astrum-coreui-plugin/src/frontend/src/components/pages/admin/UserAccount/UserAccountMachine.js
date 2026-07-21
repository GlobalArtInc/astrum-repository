
import {assign} from 'xstate';
import Axios from 'axios';
import {ExtJS, FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import {mergeDeepRight} from 'ramda';

const userAccountMachine = FormUtils.buildFormMachine({
  id: 'UserAccount',

  context: {
    data: {
      external: null,
      userId: null
    }
  },

  config: (config) => (mergeDeepRight(config, {
    states: {
      loaded: {
        on: {
          RESET: {
            cond: 'isInternal'
          }
        }
      }
    }
  }))
}).withConfig({
  actions: {
    logLoadError: ({error}) => {
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(UIStrings.USER_ACCOUNT.MESSAGES.LOAD_ERROR);
    },
    logSaveSuccess: () => {
      ExtJS.showSuccessMessage(UIStrings.USER_ACCOUNT.MESSAGES.UPDATE_SUCCESS);
    },

    validate: assign({
      validationErrors: ({data}) => ({
        firstName: ValidationUtils.validateNotBlank(data.firstName),
        lastName: ValidationUtils.validateNotBlank(data.lastName),
        email: ValidationUtils.validateNotBlank(data.email) || ValidationUtils.validateEmail(data.email),
      })
    })
  },

  guards: {
    isInternal: ({data: {external}}) => !Boolean(external)
  },

  services: {
    fetchData: () => Axios.get('service/rest/internal/ui/user'),
    saveData: ({data}) => {
      return Axios.put('service/rest/internal/ui/user', data);
    }
  }
});

export default userAccountMachine;
