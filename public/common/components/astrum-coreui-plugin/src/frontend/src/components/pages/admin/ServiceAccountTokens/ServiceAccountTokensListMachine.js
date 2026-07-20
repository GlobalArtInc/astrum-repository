
import {assign} from 'xstate';
import Axios from 'axios';
import {mergeDeepRight} from 'ramda';

import {ListMachineUtils, ExtJS} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

import {URL, canCreateToken, mapCreateError, mapRevokeError, mapRolesError} from './ServiceAccountTokensHelper';

const {SERVICE_ACCOUNT_TOKENS: {MESSAGES}} = UIStrings;

export default ListMachineUtils.buildListMachine({
  id: 'ServiceAccountTokensListMachine',
  sortableFields: ['name', 'roleId', 'createdBy', 'expiresAt', 'lastUsedAt'],
  sortField: 'name',
  config: (config) =>
    mergeDeepRight(config, {
      context: {
        createdToken: null,
        roles: [],
        rolesError: null,
      },
      states: {
        loaded: {
          on: {
            CREATE_TOKEN: {
              target: 'creating',
            },
            REVOKE_TOKEN: {
              target: 'revoking',
            },
            CLEAR_CREATED_TOKEN: {
              target: 'loading',
              actions: ['clearCreatedToken'],
            },
          },
        },
        creating: {
          invoke: {
            src: 'createToken',
            onDone: {
              target: 'loaded',
              actions: ['storeCreatedToken', 'onCreateSuccess'],
            },
            onError: {
              target: 'loaded',
              actions: ['onCreateError'],
            },
          },
        },
        revoking: {
          invoke: {
            src: 'revokeToken',
            onDone: {
              target: 'loading',
              actions: ['onRevokeSuccess'],
            },
            onError: {
              target: 'loaded',
              actions: ['onRevokeError'],
            },
          },
        },
      },
    }),
}).withConfig({
  actions: {
    setData: assign({
      data: (_, {data: [tokens]}) => tokens.data,
      pristineData: (_, {data: [tokens]}) => tokens.data,
      roles: (_, {data: [, roles]}) => roles.data,
      rolesError: (_, {data: [, roles]}) => roles.error ?? null,
    }),
    filterData: assign({
      data: ({filter, pristineData}) => pristineData.filter(({name, roleId, createdBy}) =>
          ListMachineUtils.hasAnyMatches([name, roleId, createdBy], filter)
      ),
    }),
    storeCreatedToken: assign({
      createdToken: (_, event) => ({
        rawToken: event.data?.data?.token,
        name: event.data?.data?.name,
      }),
    }),
    clearCreatedToken: assign({
      createdToken: () => null,
    }),
    onCreateSuccess: (_, event) => {
      ExtJS.showSuccessMessage(MESSAGES.CREATE_SUCCESS(event.data?.data?.name));
    },
    onCreateError: (_, event) => {
      ExtJS.showErrorMessage(mapCreateError(event));
    },
    onRevokeSuccess: (_, event) => {
      ExtJS.showSuccessMessage(MESSAGES.REVOKE_SUCCESS(event.data?.tokenName));
    },
    onRevokeError: (_, event) => {
      ExtJS.showErrorMessage(mapRevokeError(event));
    },
  },
  services: {
    fetchData: () => Axios.all([
      Axios.get(URL.serviceAccountTokensUrl),
      canCreateToken()
          ? Axios.get(URL.rolesUrl).catch((err) => ({data: [], error: mapRolesError(err)}))
          : Promise.resolve({data: []}),
    ]),
    createToken: (_, {payload}) => Axios.post(URL.serviceAccountTokensUrl, payload),
    revokeToken: async (_, {tokenId, tokenName}) => {
      await Axios.delete(URL.singleTokenUrl(tokenId));
      return {tokenId, tokenName};
    },
  },
});
