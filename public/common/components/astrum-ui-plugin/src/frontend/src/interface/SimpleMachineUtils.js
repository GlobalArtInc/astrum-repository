
import Axios from 'axios';
import {assign, createMachine} from 'xstate';
import {useMachine} from '@xstate/react';

export default function useSimpleMachine({
  id,
  url,
  initial = 'loading'
}) {
  const machine = _buildSimpleMachine(id, url, initial);

  const [current, send] = useMachine(machine, {devTools: true});

  const load = (eventPayload = {}) => send({type: 'LOAD_DATA', ...eventPayload});

  const retry = () => send({type: 'RETRY'});

  const isLoading = current.matches('loading');

  return {current, send, load, retry, isLoading};
}

/**
 * @private
 * Internal machine builder for SimpleMachine - DO NOT USE DIRECTLY.
 * This is exported only for testing purposes.
 * For production code, always use the useSimpleMachine hook.
 *
 * @param {string} id - unique identifier for the machine
 * @param {string|function} url - URL string or function that returns URL
 * @param {string} initial - initial state name
 */
export const _buildSimpleMachine = (id, url, initial) =>
  createMachine(
    {
      id,
      initial,
      context: {
        data: undefined,
        error: undefined
      },
      states: {
        loaded: {
          on: {
            LOAD_DATA: {
              target: 'loading'
            }
          }
        },
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
        }
      }
    },
    {
      actions: {
        setData: assign({
          data: (_, event) => event.data?.data
        }),
        setError: assign({
          error: (_, event) => event.data?.message
        }),
        clearError: assign({
          error: () => null
        })
      },
      services: {
        fetchData: (_, event) => Axios.get(typeof url === 'function' ? url(event) : url)
      }
    }
  );
