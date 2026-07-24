

import axios from 'axios';
import {assign, createMachine} from 'xstate';

const TEST_CONNECTION_URL = 'service/rest/internal/ui/azureblobstore/test-connection';

export default createMachine({
  id: 'AzureBlobStoreSettingsMachine',
  initial: 'editing',
  context: {
    accountName: '',
    containerName: '',
    authenticationMethod: undefined,
    accountKey: ''
  },
  states: {
    editing: {
      initial: 'untested',
      states: {
        untested: {},
        error: {},
        success: {}
      },
      on: {
        TEST_CONNECTION: 'testing'
      }
    },

    testing: {
      invoke: {
        src: 'testConnection',
        onDone: 'editing.success',
        onError: 'editing.error'
      }
    }
  },

  on: {
    UPDATE_ACCOUNT_NAME: {
      target: 'editing',
      actions: ['updateAccountName']
    },
    UPDATE_CONTAINER_NAME: {
      target: 'editing',
      actions: ['updateContainerName']
    },
    UPDATE_AUTH_METHOD: {
      target: 'editing',
      actions: ['updateAuthenticationMethod']
    },
    UPDATE_ACCOUNT_KEY: {
      target: 'editing',
      actions: ['updateAccountKey']
    }
  }
}, {
  actions: {
    updateAccountName: assign({
      accountName: (_, {accountName}) => accountName
    }),
    updateContainerName: assign({
      containerName: (_, {containerName}) => containerName
    }),
    updateAuthenticationMethod: assign({
      authenticationMethod: (_, {authenticationMethod}) => authenticationMethod
    }),
    updateAccountKey: assign({
      accountKey: (_, {accountKey}) => accountKey
    })
  },
  services: {
    testConnection: (context) => {
      if (context.blobStoreName) {
        return axios.post(TEST_CONNECTION_URL + "/" + context.blobStoreName, context)
      } else {
        return axios.post(TEST_CONNECTION_URL, context)
      }
    }
  }
});
