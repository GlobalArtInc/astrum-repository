

import { interpret } from 'xstate';
import { loggerFormMachine } from '../LoggerFormMachine';
import { Logger, LogLevel } from '../types';

const mockLogger: Logger = { name: 'org.globalart', level: 'DEBUG', override: true };

function makeMachine(
  contextOverrides: Partial<{
    isCreate: boolean;
    loggerName: string;
    name: string;
    level: LogLevel;
  }> = {},
  serviceOverrides: {
    fetchLogger?: () => Promise<Logger>;
    saveLogger?: () => Promise<void>;
  } = {}
) {
  return loggerFormMachine
    .withContext({ ...loggerFormMachine.context, ...contextOverrides })
    .withConfig({
      services: {
        fetchLogger: serviceOverrides.fetchLogger ?? (async () => mockLogger),
        saveLogger: serviceOverrides.saveLogger ?? (async () => undefined),
      },
    });
}

describe('loggerFormMachine', () => {
  describe('initial state routing via init', () => {
    it('transitions directly to form state when isCreate is true', () => {
      const machine = makeMachine({ isCreate: true });
      const service = interpret(machine);
      service.start();

      // The init state uses always transitions, so by the time start() returns
      // the machine should already be in the form state
      expect(service.getSnapshot().matches('form')).toBe(true);

      service.stop();
    });

    it('transitions to loading state when isCreate is false (edit mode)', (done) => {
      const machine = makeMachine({ isCreate: false, loggerName: 'org.globalart' });
      const service = interpret(machine);

      service.onTransition((state) => {
        if (state.matches('loading')) {
          service.stop();
          done();
        }
      });

      service.start();
    });
  });

  describe('loading → form (edit mode)', () => {
    it('loads logger data into context on successful fetch', (done) => {
      const machine = makeMachine({ isCreate: false, loggerName: 'org.globalart' });
      const service = interpret(machine);

      service.onTransition((state) => {
        if (state.matches('form')) {
          expect(state.context.name).toBe('org.globalart');
          expect(state.context.level).toBe('DEBUG');
          expect(state.context.originalLevel).toBe('DEBUG');
          service.stop();
          done();
        }
      });

      service.start();
    });

    it('transitions to fetchError on failed fetch', (done) => {
      const machine = makeMachine({ isCreate: false, loggerName: 'org.globalart' }, {
        fetchLogger: async () => { throw new Error('Not found'); },
      });
      const service = interpret(machine);

      service.onTransition((state) => {
        if (state.matches('fetchError')) {
          expect(state.context.error).toBe('Not found');
          service.stop();
          done();
        }
      });

      service.start();
    });

    it('RETRY from fetchError transitions back to loading', (done) => {
      let fetchErrorReached = false;
      const machine = makeMachine({ isCreate: false, loggerName: 'org.globalart' }, {
        fetchLogger: async () => {
          if (!fetchErrorReached) throw new Error('Failed');
          return mockLogger;
        },
      });
      const service = interpret(machine);

      service.onTransition((state) => {
        if (state.matches('fetchError') && !fetchErrorReached) {
          fetchErrorReached = true;
          service.send({ type: 'RETRY' });
        }
        if (state.matches('form') && fetchErrorReached) {
          expect(state.context.name).toBe('org.globalart');
          service.stop();
          done();
        }
      });

      service.start();
    });
  });

  describe('form interactions (create mode)', () => {
    it('SET_NAME updates name in context', () => {
      const machine = makeMachine({ isCreate: true });
      const service = interpret(machine);
      service.start();

      service.send({ type: 'SET_NAME', value: 'org.test' });

      expect(service.getSnapshot().context.name).toBe('org.test');

      service.stop();
    });

    it('SET_LEVEL updates level in context', () => {
      const machine = makeMachine({ isCreate: true });
      const service = interpret(machine);
      service.start();

      service.send({ type: 'SET_LEVEL', value: 'WARN' });

      expect(service.getSnapshot().context.level).toBe('WARN');

      service.stop();
    });

    it('SUBMIT transitions from form to saving', () => {
      // Use a slow save so we can observe the saving state
      const machine = makeMachine({ isCreate: true, name: 'org.test' }, {
        saveLogger: () => new Promise(() => {}),
      });
      const service = interpret(machine);
      service.start();

      service.send({ type: 'SUBMIT' });

      expect(service.getSnapshot().matches('saving')).toBe(true);

      service.stop();
    });
  });

  describe('saving', () => {
    it('transitions to success (final) on successful save', (done) => {
      const machine = makeMachine({ isCreate: true, name: 'org.test', level: 'DEBUG' });
      const service = interpret(machine);

      service.onTransition((state) => {
        if (state.matches('success')) {
          // success is final state
          expect(state.done).toBe(true);
          service.stop();
          done();
        }
      });

      service.start();
      service.send({ type: 'SUBMIT' });
    });

    it('transitions back to form with error on failed save', (done) => {
      const machine = makeMachine({ isCreate: true, name: 'org.test' }, {
        saveLogger: async () => { throw new Error('Permission denied'); },
      });
      const service = interpret(machine);
      let submitted = false;

      service.onTransition((state) => {
        if (state.matches('form') && submitted) {
          expect(state.context.error).toBe('Permission denied');
          service.stop();
          done();
        }
        if (state.matches('form') && !submitted) {
          submitted = true;
          service.send({ type: 'SUBMIT' });
        }
      });

      service.start();
    });
  });

  describe('context helpers', () => {
    it('SUBMIT clears any prior error', (done) => {
      // Get machine into form state with an error from a failed save
      const failOnce = (() => {
        let failed = false;
        return async () => {
          if (!failed) { failed = true; throw new Error('Save error'); }
        };
      })();

      const machine = makeMachine({ isCreate: true, name: 'org.test' }, {
        saveLogger: failOnce,
      });
      const service = interpret(machine);
      let savedError = false;

      service.onTransition((state) => {
        if (state.matches('form') && state.context.error && !savedError) {
          savedError = true;
          // Submit again to clear the error
          service.send({ type: 'SUBMIT' });
        }
        if (state.matches('saving') && savedError) {
          // error should have been cleared by SUBMIT's clearError action
          expect(state.context.error).toBeNull();
          service.stop();
          done();
        }
      });

      service.start();
      service.send({ type: 'SUBMIT' });
    });
  });
});
