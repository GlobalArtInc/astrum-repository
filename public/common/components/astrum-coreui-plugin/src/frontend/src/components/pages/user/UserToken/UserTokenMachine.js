
import {assign, createMachine} from "xstate";
import Axios from 'axios';

import {APIConstants, ExtJS} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {
  USER_TOKEN: {
    MESSAGES,
    USER_TOKEN_STATUS
  }
} = UIStrings;

const API_URL = APIConstants.REST.USER_TOKEN_TIMESTAMP;

export default createMachine({
  id: 'UserTokenMachine',
  initial: 'loading',
  context: {
    data: {},
    error: null,
    token: null
  },
  states: {
    loading: {
      invoke: {
        src: 'fetchData',
        onDone: {
          target: 'loaded',
          actions: 'setData',
        },
        onError: {
          target: 'loaded',
          actions: 'setError'
        }
      },
    },
    loaded: {
      on: {
        ACCESS: 'accessToken',
        RESET: 'resetToken',
        GENERATE: 'generateToken'
      }
    },
    accessToken: {
      invoke: {
        id: 'accessToken',
        src: 'accessToken',
        onDone: {
          target: 'showToken',
          actions: 'setToken'
        },
        onError: {
          target: 'loaded',
          actions: 'showAccessError'
        }
      },
    },
    generateToken: {
      invoke: {
        id: 'generateToken',
        src: 'generateToken',
        onDone: {
          target: 'showToken',
          actions: ['clear', 'setToken']
        },
        onError: {
          target: 'loaded',
          actions: 'showGenerateError'
        }
      },
    },
    resetToken: {
      invoke: {
        id: 'resetToken',
        src: 'resetToken',
        onDone: {
          target: 'loaded',
          actions: ['clear', 'showResetSuccess']
        },
        onError: {
          target: 'loaded',
          actions: 'showResetError'
        }
      }
    },
    showToken: {
      after: {
        60000: 'loading'
      },
      exit: 'unsetToken',
      on: {
        HIDE: {
          target: 'loading',
        }
      }
    }
  },
  on: {
    RETRY: {
      target: 'loaded'
    }
  }
}).withConfig({
  actions: {
    showResetSuccess: () => {
      ExtJS.showSuccessMessage(MESSAGES.RESET_SUCCESS)
    },
    showResetError: () => {
      ExtJS.showErrorMessage(MESSAGES.RESET_ERROR)
    },
    showAccessError: () => {
      ExtJS.showErrorMessage(MESSAGES.ACCESS_ERROR);
    },
    showGenerateError: () => {
      ExtJS.showErrorMessage(MESSAGES.GENERATE_ERROR);
    },
    setToken: assign({
      token: (_, event) => event.data.data
    }),
    unsetToken: assign({
      token: null
    }),
    setData: assign({
      data: (_, event) => event?.data?.data
    }),
    setError: assign({
      error: (_, event) => {
        if (event?.data?.response?.status === 410) {
          return USER_TOKEN_STATUS.EXPIRED_WARNING;
        }
      }
    }),
    clear: assign({
      data: {},
      error: null
    }),
  },
  services: {
    fetchData: () => Axios.get(API_URL),
    resetToken: () => ExtJS.requestAuthenticationToken(UIStrings.USER_TOKEN.MESSAGES.RESET_AUTHENTICATION)
      .then(authToken =>
          Axios.delete(`/service/rest/internal/current-user/user-token?authToken=${btoa(authToken)}`)),
    accessToken: () => ExtJS.requestAuthenticationToken(UIStrings.USER_TOKEN.MESSAGES.ACCESS_AUTHENTICATION)
      .then(authToken =>
          Axios.get(`/service/rest/internal/current-user/user-token?authToken=${btoa(authToken)}`)),
    generateToken: () => ExtJS.requestAuthenticationToken(UIStrings.USER_TOKEN.MESSAGES.GENERATE_AUTHENTICATION)
      .then(authToken =>
          Axios.post(`/service/rest/internal/current-user/user-token?authToken=${btoa(authToken)}`))
  }
});
