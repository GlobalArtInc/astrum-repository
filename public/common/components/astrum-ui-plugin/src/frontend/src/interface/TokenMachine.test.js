

import { interpret } from 'xstate';
import { awaitTransition } from '../../__jest__/xstateTestUtils';
import TokenMachine from './TokenMachine';

describe('TokenMachine', () => {
  let mockAccessToken;
  let mockResetToken;
  let mockShowAccessError;
  let mockShowResetSuccess;
  let mockShowResetError;

  beforeEach(() => {
    jest.clearAllMocks();
    jest.useFakeTimers();

    mockAccessToken = jest.fn();
    mockResetToken = jest.fn();
    mockShowAccessError = jest.fn();
    mockShowResetSuccess = jest.fn();
    mockShowResetError = jest.fn();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  const createTestMachine = () => {
    return TokenMachine.withConfig({
      services: {
        accessToken: mockAccessToken,
        resetToken: mockResetToken,
      },
      actions: {
        showAccessError: mockShowAccessError,
        showResetSuccess: mockShowResetSuccess,
        showResetError: mockShowResetError,
      },
    });
  };

  describe('access token flow', () => {
    it('initializes in idle state', () => {
      const machine = createTestMachine();
      const service = interpret(machine);
      service.start();

      expect(service.state.matches('idle')).toBe(true);
      service.stop();
    });

    it('transitions to accessToken on ACCESS event', () => {
      mockAccessToken.mockReturnValue(new Promise(() => {}));

      const machine = createTestMachine();
      const service = interpret(machine);
      service.start();

      service.send({ type: 'ACCESS' });

      expect(service.state.matches('accessToken')).toBe(true);
      service.stop();
    });

    it('transitions to showToken on successful access', async () => {
      mockAccessToken.mockResolvedValue({ data: 'test-token-123' });

      const machine = createTestMachine();

      await awaitTransition(
        machine,
        'showToken',
        (state) => {
          expect(state.context.token).toBe('test-token-123');
        },
        (service) => service.send({ type: 'ACCESS' })
      );
    });

    it('transitions to idle and shows error on failed access', async () => {
      mockAccessToken.mockRejectedValue(new Error('Access denied'));

      const machine = createTestMachine();
      let hasSeenAccessToken = false;

      await awaitTransition(
        machine,
        (state) => {
          if (state.matches('accessToken')) {
            hasSeenAccessToken = true;
          }
          return state.matches('idle') && hasSeenAccessToken;
        },
        () => {
          expect(mockShowAccessError).toHaveBeenCalled();
        },
        (service) => service.send({ type: 'ACCESS' })
      );
    });
  });

  describe('reset token flow', () => {
    it('transitions to resetToken on RESET event', () => {
      mockResetToken.mockReturnValue(new Promise(() => {}));

      const machine = createTestMachine();
      const service = interpret(machine);
      service.start();

      service.send({ type: 'RESET' });

      expect(service.state.matches('resetToken')).toBe(true);
      service.stop();
    });

    it('transitions to idle and shows success on successful reset', async () => {
      mockResetToken.mockResolvedValue({ success: true });

      const machine = createTestMachine();
      let hasSeenResetToken = false;

      await awaitTransition(
        machine,
        (state) => {
          if (state.matches('resetToken')) {
            hasSeenResetToken = true;
          }
          return state.matches('idle') && hasSeenResetToken;
        },
        () => {
          expect(mockShowResetSuccess).toHaveBeenCalled();
          expect(mockShowResetError).not.toHaveBeenCalled();
        },
        (service) => service.send({ type: 'RESET' })
      );
    });

    it('transitions to idle and shows error on failed reset', async () => {
      mockResetToken.mockRejectedValue(new Error('Reset failed'));

      const machine = createTestMachine();
      let hasSeenResetToken = false;

      await awaitTransition(
        machine,
        (state) => {
          if (state.matches('resetToken')) {
            hasSeenResetToken = true;
          }
          return state.matches('idle') && hasSeenResetToken;
        },
        () => {
          expect(mockShowResetError).toHaveBeenCalled();
          expect(mockShowResetSuccess).not.toHaveBeenCalled();
        },
        (service) => service.send({ type: 'RESET' })
      );
    });
  });

  describe('showToken state', () => {
    it('clears token after 60 second timeout', async () => {
      mockAccessToken.mockResolvedValue({ data: 'timeout-token' });

      const machine = createTestMachine();

      await awaitTransition(
        machine,
        (state) => {
          if (state.matches('showToken')) {
            jest.advanceTimersByTime(60000);
          }
          return state.matches('idle') && state.context.token === null;
        },
        (state) => {
          expect(state.context.token).toBeNull();
        },
        (service) => service.send({ type: 'ACCESS' })
      );
    });

    it('clears token on HIDE event', async () => {
      mockAccessToken.mockResolvedValue({ data: 'hide-token' });

      const machine = createTestMachine();
      let hasSeenShowToken = false;

      await awaitTransition(
        machine,
        (state, service) => {
          if (state.matches('showToken') && !hasSeenShowToken) {
            hasSeenShowToken = true;
            service.send({ type: 'HIDE' });
          }
          return state.matches('idle') && hasSeenShowToken;
        },
        (state) => {
          expect(state.context.token).toBeNull();
        },
        (service) => service.send({ type: 'ACCESS' })
      );
    });
  });

  describe('global RETRY event', () => {
    it('transitions to idle from any state', () => {
      mockAccessToken.mockReturnValue(new Promise(() => {}));

      const machine = createTestMachine();
      const service = interpret(machine);
      service.start();

      service.send({ type: 'ACCESS' });
      expect(service.state.matches('accessToken')).toBe(true);

      service.send({ type: 'RETRY' });
      expect(service.state.matches('idle')).toBe(true);

      service.stop();
    });
  });
});
