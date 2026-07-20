
import {assign, createMachine} from 'xstate';
import Axios from 'axios';
import {ExtJS, APIConstants} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {RECOVERY_MODE} = UIStrings;

const {
  RECOVERY_MODE: RECOVERY_MODE_PUBLIC_API
} = APIConstants.REST.PUBLIC;

const {
  RECOVERY_MODE: RECOVERY_MODE_UI_API
} = APIConstants.REST.INTERNAL;

export default createMachine(
{
  id: 'RecoveryMode',
  initial: 'loading',

  context: {
    data: {},
    error: null
  },

  states: {
    loading: {
      invoke: {
        src: 'fetchData',
        onDone: {
          target: 'loaded',
          actions: ['setData']
        },
        onError: {
          target: 'loadError',
          actions: ['logLoadError']
        }
      }
    },
    loaded: {
      on: {
        ENABLE: {
          target: 'setEnabled',
          actions: ['enableRecoveryMode']
        },
        DISABLE: [
          {
            target: 'confirmingDisable',
            cond: 'hasUnexecutedPlans'
          },
          {
            target: 'setDisabled',
            actions: ['disableRecoveryMode']
          }
        ]
      }
    },
    confirmingDisable: {
      on: {
        CONFIRM: {
          target: 'setDisabled',
          actions: ['disableRecoveryMode']
        },
        CANCEL: {
          target: 'loaded'
        }
      }
    }  ,
    setEnabled: {
      invoke: {
        src: 'enableRecoveryMode',
        onDone: {
          target: 'loaded',
          actions: ['logSaveSuccess']
        },
        onError: {
          target: 'loaded',
          actions: ['logSaveError']
        }
      }
    },
    setDisabled: {
      invoke: {
        src: 'disableRecoveryMode',
        onDone: {
          target: 'loaded',
          actions: ['logSaveSuccess']
        },
        onError: {
          target: 'loaded',
          actions: ['logSaveError']
        }
      }
    },
    loadError: {
      on: {
        RETRY: {
          target: 'loading'
        }
      }
    }
  },

},
{
  actions: {
    setData: assign({
      data: (_, {data}) => data.data,
      pristineData: (_, {data}) => data.data
    }),
    enableRecoveryMode: assign({
      data: (context) => ({
        ...context.data,
        enabled: true
      })
    }),
    disableRecoveryMode: assign({
      data: (context) => ({
        ...context.data,
        enabled: false
      })
    }),
    logError: (_, event) => console.error('Failed to load recovery mode', event),

    logLoadError: (_, {error}) => {
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(RECOVERY_MODE.MESSAGES.LOAD_ERROR);
    },

    logSaveError: (_, {error}) => {
      if (error) {
        console.error(error);
      }
      ExtJS.showErrorMessage(RECOVERY_MODE.MESSAGES.SAVE_ERROR);
    },

    logSaveSuccess: (context) => {
      ExtJS.showSuccessMessage(RECOVERY_MODE.MESSAGES.SAVE_SUCCESS);
      ExtJS.state().setValue('recovery.mode.enabled', context?.data?.enabled);
    },
  },

  guards: {
    hasUnexecutedPlans: (context) => {
      return context?.data?.unexecutedPlans;
    }
  },

  services: {
    fetchData: () => Axios.get(RECOVERY_MODE_UI_API),
    enableRecoveryMode: () => Axios.post(RECOVERY_MODE_PUBLIC_API),
    disableRecoveryMode: () => Axios.delete(RECOVERY_MODE_PUBLIC_API)
  }
});
