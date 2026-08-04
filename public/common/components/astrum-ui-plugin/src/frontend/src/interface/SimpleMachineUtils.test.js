

import { interpret } from 'xstate';
import { renderHook, waitFor } from '@testing-library/react';
import Axios from 'axios';
import { awaitTransition } from '../../__jest__/xstateTestUtils';
import useSimpleMachine, { _buildSimpleMachine } from './SimpleMachineUtils';

jest.mock('axios');

describe('SimpleMachineUtils', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('_buildSimpleMachine', () => {
    it('initializes in the specified initial state', () => {
      const machine = _buildSimpleMachine('test-machine', '/api/test', 'loading');
      const service = interpret(machine);
      service.start();

      expect(service.state.matches('loading')).toBe(true);
      service.stop();
    });

    it('transitions to loaded on successful fetch', async () => {
      const mockData = { data: { items: ['item1', 'item2'] } };
      Axios.get.mockResolvedValue(mockData);

      const machine = _buildSimpleMachine('test-machine', '/api/test', 'loading');

      await awaitTransition(machine, 'loaded', (state) => {
        expect(state.context.data).toEqual(mockData.data);
        expect(Axios.get).toHaveBeenCalledWith('/api/test');
      });
    });

    it('transitions to error on failed fetch', async () => {
      Axios.get.mockRejectedValue(new Error('Network error'));

      const machine = _buildSimpleMachine('test-machine', '/api/test', 'loading');

      await awaitTransition(machine, 'error', (state) => {
        expect(state.context.error).toBe('Network error');
      });
    });

    it('reloads data on LOAD_DATA event', async () => {
      Axios.get.mockResolvedValue({ data: { items: [] } });

      const machine = _buildSimpleMachine('test-machine', '/api/test', 'loaded');
      let loadedCount = 0;

      await awaitTransition(
        machine,
        (state, service) => {
          if (state.matches('loaded')) {
            loadedCount++;
            if (loadedCount === 1) {
              service.send({ type: 'LOAD_DATA' });
            }
          }
          return state.matches('loaded') && loadedCount === 2;
        },
        () => {
          expect(Axios.get).toHaveBeenCalledTimes(1);
        }
      );
    });

    it('retries after error on RETRY event', async () => {
      Axios.get
        .mockRejectedValueOnce(new Error('Failed'))
        .mockResolvedValueOnce({ data: { success: true } });

      const machine = _buildSimpleMachine('test-machine', '/api/test', 'loading');
      let hasSeenError = false;

      await awaitTransition(
        machine,
        (state, service) => {
          if (state.matches('error') && !hasSeenError) {
            hasSeenError = true;
            service.send({ type: 'RETRY' });
          }
          return state.matches('loaded') && hasSeenError;
        },
        (state) => {
          expect(state.context.data).toEqual({ success: true });
          expect(state.context.error).toBeNull();
        }
      );
    });

    it('supports URL as a function', async () => {
      Axios.get.mockResolvedValue({ data: { result: 'success' } });

      const urlFunction = jest.fn(() => '/api/dynamic-url');
      const machine = _buildSimpleMachine('test-machine', urlFunction, 'loading');

      await awaitTransition(machine, 'loaded', () => {
        expect(urlFunction).toHaveBeenCalled();
        expect(Axios.get).toHaveBeenCalledWith('/api/dynamic-url');
      });
    });

    it('maintains data in context when error occurs', async () => {
      Axios.get
        .mockResolvedValueOnce({ data: { value: 'first' } })
        .mockRejectedValueOnce(new Error('Error occurred'));

      const machine = _buildSimpleMachine('test-machine', '/api/test', 'loading');
      let firstLoadComplete = false;

      await awaitTransition(
        machine,
        (state, service) => {
          if (state.matches('loaded') && !firstLoadComplete) {
            firstLoadComplete = true;
            service.send({ type: 'LOAD_DATA' });
          }
          return state.matches('error') && firstLoadComplete;
        },
        (state) => {
          expect(state.context.data).toEqual({ value: 'first' });
          expect(state.context.error).toBe('Error occurred');
        }
      );
    });
  });

  describe('useSimpleMachine hook', () => {
    it('returns the expected API shape', () => {
      Axios.get.mockResolvedValue({ data: {} });

      const { result } = renderHook(() =>
        useSimpleMachine({
          id: 'test-hook',
          url: '/api/test',
          initial: 'loading'
        })
      );

      expect(result.current).toHaveProperty('current');
      expect(result.current).toHaveProperty('send');
      expect(result.current).toHaveProperty('load');
      expect(result.current).toHaveProperty('retry');
      expect(result.current).toHaveProperty('isLoading');
    });

    it('retry() recovers from error state', async () => {
      Axios.get
        .mockRejectedValueOnce(new Error('Test error'))
        .mockResolvedValueOnce({ data: { success: true } });

      const { result } = renderHook(() =>
        useSimpleMachine({
          id: 'test-hook',
          url: '/api/test',
          initial: 'loading'
        })
      );

      await waitFor(() => {
        expect(result.current.current.matches('error')).toBe(true);
      });

      result.current.retry();

      await waitFor(() => {
        expect(result.current.current.matches('loaded')).toBe(true);
      });

      expect(result.current.current.context.data).toEqual({ success: true });
    });
  });
});
