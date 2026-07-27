
import {assign} from 'xstate';
import {mergeDeepRight} from 'ramda';

import {APIConstants, FormUtils, ValidationUtils, ExtAPIUtils} from '@globalart/astrum-ui-plugin';

const {
  ACTION,
  METHODS: {VERIFY_LOGIN}
} = APIConstants.EXT.LDAP;

const INITIAL_DATA = {username: '', password: ''};

export default FormUtils.buildFormMachine({
  id: 'LdapVerifyLoginMachine',
  initial: 'loaded',
  config: (config) =>
    mergeDeepRight(config, {
      context: {
        data: INITIAL_DATA,
        pristineData: INITIAL_DATA
      },
      states: {
        loaded: {
          on: {
            VERIFY: {
              target: 'verifying'
            }
          }
        },
        verifying: {
          entry: 'clearResult',
          invoke: {
            src: 'verifyLogin',
            onDone: {
              target: 'loaded',
              actions: 'setSuccess'
            },
            onError: {
              target: 'loaded',
              actions: 'setError'
            }
          }
        }
      }
    })
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data}) => ({
        username: ValidationUtils.validateNotBlank(data.username),
        password: ValidationUtils.validateNotBlank(data.password)
      })
    }),
    setSuccess: assign({
      success: true
    }),
    setError: assign({
      error: (_, evt) => evt.data.message
    }),
    clearResult: assign({
      success: false,
      error: null
    })
  },
  services: {
    verifyLogin: async (context) => {
      const {
        ldapConfig,
        data: {username, password}
      } = context;
      const config = {
        ...ldapConfig,
        protocol: ldapConfig.protocol.toLowerCase()
      };
      const response = await ExtAPIUtils.extAPIRequest(ACTION, VERIFY_LOGIN, {
        data: [config, btoa(username), btoa(password)]
      });
      return ExtAPIUtils.checkForError(response) || response;
    }
  }
});
