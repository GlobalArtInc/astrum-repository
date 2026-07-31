
import {assign, createMachine} from 'xstate';

export default createMachine(
    {
      initial: 'idle',
      context: {
        userToken: null,
      },
      states: {
        idle: {
          on: {
            ACCESS: 'accessToken',
            RESET: 'resetToken',
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
              target: 'idle',
              actions: 'showAccessError'
            }
          },
        },
        resetToken: {
          invoke: {
            id: 'resetToken',
            src: 'resetToken',
            onDone: {
              target: 'idle',
              actions: 'showResetSuccess'
            },
            onError: {
              target: 'idle',
              actions: 'showResetError'
            }
          }
        },
        showToken: {
          after: {
            60000: 'idle'
          },
          exit: 'unsetToken',
          on: {
            ACCESS: 'accessToken',
            RESET: 'resetToken',
            HIDE: {
              target: 'idle',
              actions: 'unsetToken'
            }
          }
        }
      },
      on: {
        RETRY: {
          target: 'idle'
        }
      }
    },
    {
      actions: {
        showResetSuccess: () => {
          console.error('showResetSuccess function must be overridden in order to show the appropriate message');
        },
        showResetError: () => {
          console.error('showResetError function must be overridden in order to show the appropriate message');
        },
        showAccessError: () => {
          console.error('showAccessError function must be overridden in order to show the appropriate message');
        },
        setToken: assign({
          token: (_, event) => event.data.data
        }),
        unsetToken: assign({
          token: null
        })
      }
    }
);
