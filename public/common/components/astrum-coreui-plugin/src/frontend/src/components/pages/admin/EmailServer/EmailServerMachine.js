
import {assign, spawn} from 'xstate';
import {isNil, mergeDeepRight} from 'ramda';
import {
  FormUtils,
  APIConstants,
  ValidationUtils,
  ExtAPIUtils,
} from '@globalart/astrum-ui-plugin';
import EmailVerifyServerMachine from './EmailVerifyServerMachine';

const {
  EXT: {
    EMAIL_SERVER: {ACTION, METHODS},
  },
} = APIConstants;

export default FormUtils.buildFormMachine({
  id: 'EmailServerMachine',
  config: (config) =>
    mergeDeepRight(config, {
      context: {
        emailVerifyServer: null,
      },
      states: {
        loaded: {
          entry: [...config.states.loaded.entry, 'initActor'],
          on: {
            UPDATE: {
              target: 'loaded',
              actions: ['update', 'updateChild'],
              internal: false,
            },
            UPDATE_AND_CLEAR_PASSWORD: {
              target: 'loaded',
              actions: ['update', 'updateChild', 'clearPassword'],
              internal: false,
            },
          },
        },
      },
    }),
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data: {host, port, fromAddress}}) => {
        return {
          host:
            ValidationUtils.validateNotBlank(host) ||
            ValidationUtils.validateHost(host),
          port:
            ValidationUtils.validateNotBlank(port) ||
            ValidationUtils.isInRange({
              value: port,
              min: 1,
              max: 65535,
              allowDecimals: false,
            }),
          fromAddress:
            ValidationUtils.validateNotBlank(fromAddress) ||
            ValidationUtils.validateEmail(fromAddress),
        };
      },
    }),
    initActor: assign({
      emailVerifyServer: (context) => {
        if (!isNil(context.emailVerifyServer)) {
          return context.emailVerifyServer;
        }

        const data = {...context.data, email: ''};

        return spawn(
          EmailVerifyServerMachine.withContext({
            data,
            pristineData: data,
            testResult: null,
          }),
          'emailVerifyServer'
        );
      },
    }),
    updateChild: assign((context) =>
      context.emailVerifyServer.send({
        type: 'UPDATE_DATA',
        data: context.data,
      })
    ),
    setData: assign({
      data: (_, event) => event.data,
      pristineData: (_, event) => event.data,
    }),
    clearPassword: assign({
      data: (ctx) => ({
        ...ctx.data,
        password: '',
      }),
    }),
  },
  services: {
    fetchData: async () => {
      const response = await ExtAPIUtils.extAPIRequest(ACTION, METHODS.READ);

      return (
        ExtAPIUtils.checkForError(response) ||
        ExtAPIUtils.extractResult(response)
      );
    },
    saveData: async ({data}) => {
      const response = await ExtAPIUtils.extAPIRequest(ACTION, METHODS.UPDATE, {
        data: [data],
      });

      return ExtAPIUtils.checkForError(response) || response;
    },
  },
});
