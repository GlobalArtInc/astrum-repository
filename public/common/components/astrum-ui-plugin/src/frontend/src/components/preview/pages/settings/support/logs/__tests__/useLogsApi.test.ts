

import { renderHook, act, } from '@testing-library/react';

import { useLogsApi } from '../useLogsApi';
import { LOGS_API } from '../types';

const mockRestClient = {
  get: jest.fn(),
  post: jest.fn(),
  put: jest.fn(),
  delete: jest.fn(),
};

// Mock the local API module used by useLogsApi
jest.mock('../../../../../../../interface/api', () => ({
  restClient: {
    get: (...args: unknown[]) => mockRestClient.get(...args),
    post: (...args: unknown[]) => mockRestClient.post(...args),
    put: (...args: unknown[]) => mockRestClient.put(...args),
    delete: (...args: unknown[]) => mockRestClient.delete(...args),
  },
  parseApiError: jest.fn((err) => ({
    message: err?.response?.data?.message || err?.message || 'An error occurred',
    status: err?.response?.status,
  })),
}));

describe('useLogsApi', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('fetchLogs', () => {
    it('fetches log files successfully', async () => {
      const mockLogs = [
        { fileName: 'astrum.log', size: 1024, lastModified: 1699999999999 },
        { fileName: 'request.log', size: 2048, lastModified: 1699888888888 },
      ];
      mockRestClient.get.mockResolvedValueOnce(mockLogs);

      const { result } = renderHook(() => useLogsApi());

      let logs: any;
      await act(async () => {
        logs = await result.current.fetchLogs();
      });

      expect(mockRestClient.get).toHaveBeenCalledWith(LOGS_API.LIST);
      expect(logs).toEqual(mockLogs);
    });

    it('handles fetch error', async () => {
      const errorMessage = 'Network Error';
      mockRestClient.get.mockRejectedValueOnce(new Error(errorMessage));

      const { result } = renderHook(() => useLogsApi());

      await act(async () => {
        await expect(result.current.fetchLogs()).rejects.toThrow();
      });

      expect(result.current.error).toBe(errorMessage);
    });

    it('returns empty array when response is not an array', async () => {
      mockRestClient.get.mockResolvedValueOnce(null);

      const { result } = renderHook(() => useLogsApi());

      let logs: any;
      await act(async () => {
        logs = await result.current.fetchLogs();
      });

      expect(logs).toEqual([]);
    });
  });

  describe('fetchLogContent', () => {
    it('fetches log content successfully', async () => {
      const mockContent = '2024-01-01 12:00:00 INFO  [main] - Test log message';
      mockRestClient.get.mockResolvedValueOnce(mockContent);

      const { result } = renderHook(() => useLogsApi());

      let content: string = '';
      await act(async () => {
        content = await result.current.fetchLogContent('astrum.log', -25000);
      });

      expect(mockRestClient.get).toHaveBeenCalledWith(
        LOGS_API.VIEW('astrum.log'),
        {
          params: { bytesCount: -25000 },
          headers: { Accept: 'text/plain' },
        }
      );
      expect(content).toBe(mockContent);
    });

    it('handles fetch content error', async () => {
      const errorMessage = 'Failed to load log';
      mockRestClient.get.mockRejectedValueOnce(new Error(errorMessage));

      const { result } = renderHook(() => useLogsApi());

      await act(async () => {
        await expect(result.current.fetchLogContent('astrum.log')).rejects.toThrow(errorMessage);
      });
    });
  });

  describe('insertMark', () => {
    it('inserts mark successfully', async () => {
      mockRestClient.post.mockResolvedValueOnce({});

      const { result } = renderHook(() => useLogsApi());

      await act(async () => {
        await result.current.insertMark('TEST_MARK');
      });

      expect(mockRestClient.post).toHaveBeenCalledWith(
        LOGS_API.MARK,
        'TEST_MARK',
        { headers: { 'Content-Type': 'text/plain' } }
      );
    });

    it('handles insert mark error', async () => {
      const errorMessage = 'Failed to insert mark';
      mockRestClient.post.mockRejectedValueOnce(new Error(errorMessage));

      const { result } = renderHook(() => useLogsApi());

      await act(async () => {
        await expect(result.current.insertMark('TEST')).rejects.toThrow();
      });

      expect(result.current.error).toBe(errorMessage);
    });
  });

  describe('getDownloadUrl', () => {
    it('returns correct download URL', () => {
      const { result } = renderHook(() => useLogsApi());

      const url = result.current.getDownloadUrl('astrum.log');

      expect(url).toBe('/service/rest/internal/logging/logs/astrum.log');
    });

    it('encodes special characters in filename', () => {
      const { result } = renderHook(() => useLogsApi());

      const url = result.current.getDownloadUrl('test log file.log');

      expect(url).toBe('/service/rest/internal/logging/logs/test%20log%20file.log');
    });

    it('preserves task log directory separators in URLs', () => {
      const { result } = renderHook(() => useLogsApi());

      const url = result.current.getDownloadUrl('tasks/component.normalize.version-20260605161637239.log');

      expect(url).toBe('/service/rest/internal/logging/logs/tasks/component.normalize.version-20260605161637239.log');
    });
  });

  describe('setError', () => {
    it('can set and clear error', async () => {
      const { result } = renderHook(() => useLogsApi());

      act(() => {
        result.current.setError('Test error');
      });

      expect(result.current.error).toBe('Test error');

      act(() => {
        result.current.setError(null);
      });

      expect(result.current.error).toBeNull();
    });
  });
});
