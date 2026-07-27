
import {assign, createMachine} from 'xstate';
import {APIConstants, ExtAPIUtils} from '@globalart/astrum-ui-plugin';

const {
  ACTION,
  METHODS: {VERIFY_USER_MAPPING}
} = APIConstants.EXT.LDAP;

export default createMachine(
  {
    id: 'LdapVerifyUserMappingMachine',
    initial: 'loading',
    states: {
      loaded: {
        on: {
          SHOW_ITEM: {
            target: 'showingItem',
            actions: 'setItem'
          },
          RETRY: {
            target: 'loading'
          }
        }
      },
      loading: {
        id: 'loading',
        entry: 'clearError',
        invoke: {
          src: 'fetchData',
          onDone: {
            target: 'loaded',
            actions: 'setData'
          },
          onError: {
            target: 'loaded',
            actions: 'setError'
          }
        }
      },
      showingItem: {
        on: {
          SHOW_LIST: {
            target: 'loaded'
          }
        }
      }
    },
    on: {
      CANCEL: {
        actions: 'onCancel'
      }
    }
  },
  {
    actions: {
      setData: assign({
        data: (_, evt) => evt.data
      }),
      setItem: assign({
        item: (ctx, evt) => ctx.data[evt.itemIndex]
      }),
      setError: assign({
        error: (_, evt) => evt.data.message
      }),
      clearError: assign({
        error: () => null
      })
    },
    services: {
      fetchData: async (context) => {
        const {
          ldapConfig,
          ldapConfig: {protocol, groupType}
        } = context;
        const config = {
          ...ldapConfig,
          protocol: protocol.toLowerCase(),
          groupType: groupType.toLowerCase()
        };
        const response = await ExtAPIUtils.extAPIRequest(ACTION, VERIFY_USER_MAPPING, {
          data: [config]
        });
        return ExtAPIUtils.checkForError(response) || ExtAPIUtils.extractResult(response, []);
      }
    }
  }
);
