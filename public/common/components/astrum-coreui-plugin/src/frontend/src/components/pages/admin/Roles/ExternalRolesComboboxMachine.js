
import Axios from 'axios';
import {assign, Machine} from 'xstate';

import {URL} from './RolesHelper';

const {getLdapRolesUrl} = URL;

const EMPTY_CONTEXT = {
  query: '',
  data: [],
  error: null,
};

export default Machine(
    {
      id: 'ExternalRolesCombobox',
      initial: 'loaded',
      context: EMPTY_CONTEXT,
      states: {
        loaded: {},
        loading: {
          invoke: {
            src: 'fetchData',
            onDone: [
              {
                target: 'loaded',
                actions: 'setData'
              }
            ],
            onError: {
              target: 'error',
              actions: ['setError']
            }
          }
        },
        error: {
          on: {
            RETRY: {
              target: 'loading',
              actions: ['clearError']
            }
          }
        },
        debouncing: {
          after: {
            500: {
              target: 'loading',
              cond: 'meetsCharacterLimit',
            }
          }
        }
      },
      on: {
        SET_QUERY: [
          {
            target: 'debouncing',
            cond: 'doesNotMeetCharacterLimit' && 'isLdap',
            actions: ['resetData', 'setQuery'],
          },
          {
            target: 'debouncing',
            cond: 'meetsCharacterLimit' && 'isLdap',
            actions: ['setQuery'],
          },
          {
            target: 'loaded',
            cond: 'isNotLdap',
            actions: ['setQuery'],
          }
        ],
        UPDATE_TYPE: {
          target: 'loaded',
          actions: ['updateType'],
        },
        UPDATE_LDAP_LIMIT: {
          target: 'loaded',
          actions: ['updateLdapCharacterLimit'],
        },
      }
    },
    {
      actions: {
        setData: assign({
          data: (_, event) => {
            return event.data?.data?.map(it => ({id: it.id, displayName: it.name})) || [];
          }
        }),
        updateType: assign({
          externalRoleType: (_, {externalRoleType}) => externalRoleType,
          ...EMPTY_CONTEXT,
        }),
        updateLdapCharacterLimit: assign({
          ldapQueryCharacterLimit: (_, {ldapQueryCharacterLimit}) => ldapQueryCharacterLimit,
        }),
        resetData: assign({
          data: [],
        }),
        setQuery: assign({
          query: (_, {query}) => query,
        }),
        setError: assign({
          error: (_, event) => event.data?.message,
        }),
        clearError: assign({
          error: () => null,
        })
      },
      guards: {
        doesNotMeetCharacterLimit: ({ _, query, ldapQueryCharacterLimit}) => {
          return query.length < ldapQueryCharacterLimit;
        },
        meetsCharacterLimit: ({ _, query, ldapQueryCharacterLimit}) => {
          return query.length >= ldapQueryCharacterLimit;
        },
        isLdap: ({ _, externalRoleType}) => {
          return externalRoleType.toLowerCase() === 'ldap';
        },
        isNotLdap: ({ _, externalRoleType}) => {
          return externalRoleType.toLowerCase() !== 'ldap';
        },
      },
      services: {
        fetchData: ({externalRoleType, query}) => {
          if (externalRoleType.toLowerCase() === 'ldap') {
            return Axios.get(getLdapRolesUrl(query, externalRoleType));
          }
        },
      }
    }
);
