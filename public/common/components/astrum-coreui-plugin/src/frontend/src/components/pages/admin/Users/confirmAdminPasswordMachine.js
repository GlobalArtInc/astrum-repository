
import {assign, sendParent} from 'xstate';
import {mergeDeepRight} from 'ramda';

import {ExtJS, FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

export default FormUtils.buildFormMachine({
  id: 'confirmAdminPasswordMachine',
  initial: 'loaded',

  stateAfterSave: 'done',

  config: (config) =>
    mergeDeepRight(config, {
      context: {
        data: {
          adminPassword: '',
        },
        pristineData: {
          adminPassword: '',
        },
      },

      states: {
        done: {
          type: 'final',
        },
      },

      on: {
        CANCEL: {
          actions: sendParent('CANCEL'),
        },
      },
    }),
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data: {adminPassword}}, event) => {
        if (!event?.data?.success && event?.data?.message) {
          return {
            adminPassword: event.data.message,
          };
        }

        return {
          adminPassword: ValidationUtils.validateNotBlank(adminPassword),
        };
      },
    }),
  },
  services: {
    saveData: async ({data: {adminPassword}}) => {
      const adminUserId = ExtJS.state().getUser().id;
      const result = await ExtJS.fetchAuthenticationToken(
        adminUserId,
        adminPassword
      );

      return result.success
        ? Promise.resolve(result.success)
        : Promise.reject(result);
    },
  },
});
