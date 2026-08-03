

import { interpret } from 'xstate';

/**
 * Waits for a state machine to reach a target state, then runs assertions.
 * Automatically handles service cleanup and error handling to prevent hanging tests.
 *
 * @param {Object} machine - The XState machine to interpret
 * @param {string|Function} stateMatcher - State name string or function (state, service) => boolean
 * @param {Function} test - Callback that receives (state, service) and runs assertions
 * @param {Function} [setup] - Optional callback that receives service for sending initial events
 * @returns {Promise} Resolves when assertions pass, rejects on assertion errors
 *
 * @example
 * // Simple state match
 * await awaitTransition(machine, 'loaded', (state) => {
 *   expect(state.context.data).toEqual(expected);
 * });
 *
 * @example
 * // Complex state match with setup
 * await awaitTransition(
 *   machine,
 *   (state) => state.matches('loaded') && state.context.data,
 *   (state) => { expect(state.context.data).toBeDefined(); },
 *   (service) => service.send({ type: 'LOAD' })
 * );
 */
export const awaitTransition = (machine, stateMatcher, test, setup) => {
  return new Promise((resolve, reject) => {
    const service = interpret(machine).onTransition((state) => {
      try {
        const isTargetState = typeof stateMatcher === 'string'
          ? state.matches(stateMatcher)
          : stateMatcher(state, service);

        if (isTargetState) {
          test(state, service);
          service.stop();
          resolve();
        }
      } catch (error) {
        service.stop();
        reject(error);
      }
    });
    service.start();
    if (setup) {
      setup(service);
    }
  });
};
