
import {assign} from 'xstate';
import {mergeDeepRight} from 'ramda';
import {
  FormUtils,
  APIConstants,
  ValidationUtils,
  ExtAPIUtils,
} from '@globalart/astrum-ui-plugin';

const {
  EXT: {
    EMAIL_SERVER: {ACTION, METHODS},
  },
} = APIConstants;

export default FormUtils.buildFormMachine({
  id: 'EmailVerifyServerMachine',
  initial: 'loaded',
  config: (config) =>
    mergeDeepRight(config, {
      states: {
        loaded: {
          on: {
            UPDATE_DATA: {
              actions: 'updateData',
            },
          },
        },
      },
    }),
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data}) => ({
        email:
          ValidationUtils.notBlank(data.email) &&
          ValidationUtils.validateEmail(data.email),
      }),
    }),
    logSaveSuccess: () => {},
    logLoadError: () => {},
    setSavedData: assign({
      isTouched: () => ({}),
      testResult: (_, event) => event.data.data.result.success,
    }),
    setSaveError: assign({
      isTouched: () => ({}),
      testResult: () => false,
    }),
    updateData: assign((context, {data}) => mergeDeepRight(context, {data})),
  },
  services: {
    saveData: async ({data: {email, ...config}}) => {
      const request = config;

      const response = await ExtAPIUtils.extAPIRequest(ACTION, METHODS.VERIFY, {
        data: [request, email],
      });

      ExtAPIUtils.checkForError(response);

      return response;
    },
  },
});
