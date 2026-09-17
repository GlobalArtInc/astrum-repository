

import { renderHook, act, } from '@testing-library/react';

// Mock the REST API from the relative path used by the source
jest.mock('../../../../../../../interface/api', () => ({
  ...jest.requireActual('../../../../../../../interface/api'),
  restClient: {
    get: jest.fn(),
    post: jest.fn(),
    put: jest.fn(),
    delete: jest.fn(),
  },
  parseApiError: jest.fn((err) => ({
    message: err?.response?.data?.message || err?.message || 'An error occurred',
    status: err?.response?.status,
  })),
}));

import { useRealmsApi } from '../useRealmsApi';
import { restClient, parseApiError } from '../../../../../../../interface/api';

// Get mock references - use type assertions
const mockedRestClient = restClient as any;
const mockedParseApiError = parseApiError as any;

describe('useRealmsApi', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Reset parseApiError mock to default behavior
    mockedParseApiError.mockImplementation((err) => ({
      message: err?.response?.data?.message || err?.message || 'An error occurred',
      status: err?.response?.status,
    }));
  });

  describe('fetchAvailableRealms', () => {
    it('fetches available realms successfully', async () => {
      const mockRealms = [
        { id: 'AstrumAuthenticatingRealm', name: 'Local Authenticating Realm' },
        { id: 'LdapRealm', name: 'LDAP Realm' },
      ];

      mockedRestClient.get.mockResolvedValueOnce(mockRealms);

      const { result } = renderHook(() => useRealmsApi());

      let realms;
      await act(async () => {
        realms = await result.current.fetchAvailableRealms();
      });

      expect(mockedRestClient.get).toHaveBeenCalledWith('service/rest/v1/security/realms/available');
      expect(realms).toEqual(mockRealms);
    });

    it('returns empty array when response is not an array', async () => {
      mockedRestClient.get.mockResolvedValueOnce(null);

      const { result } = renderHook(() => useRealmsApi());

      let realms;
      await act(async () => {
        realms = await result.current.fetchAvailableRealms();
      });

      expect(realms).toEqual([]);
    });

    it('throws error when request fails', async () => {
      const mockError = new Error('Network error');
      mockedRestClient.get.mockRejectedValueOnce(mockError);
      mockedParseApiError.mockReturnValueOnce({ message: 'Network error', status: undefined });

      const { result } = renderHook(() => useRealmsApi());

      await expect(result.current.fetchAvailableRealms()).rejects.toThrow('Network error');
    });
  });

  describe('fetchActiveRealmIds', () => {
    it('fetches active realm IDs successfully', async () => {
      const mockActiveIds = ['AstrumAuthenticatingRealm', 'AstrumAuthorizingRealm'];

      mockedRestClient.get.mockResolvedValueOnce(mockActiveIds);

      const { result } = renderHook(() => useRealmsApi());

      let activeIds;
      await act(async () => {
        activeIds = await result.current.fetchActiveRealmIds();
      });

      expect(mockedRestClient.get).toHaveBeenCalledWith('service/rest/v1/security/realms/active');
      expect(activeIds).toEqual(mockActiveIds);
    });

    it('returns empty array when response is not an array', async () => {
      mockedRestClient.get.mockResolvedValueOnce(null);

      const { result } = renderHook(() => useRealmsApi());

      let activeIds;
      await act(async () => {
        activeIds = await result.current.fetchActiveRealmIds();
      });

      expect(activeIds).toEqual([]);
    });

    it('throws error when request fails', async () => {
      const mockError = new Error('Network error');
      mockedRestClient.get.mockRejectedValueOnce(mockError);
      mockedParseApiError.mockReturnValueOnce({ message: 'Network error', status: undefined });

      const { result } = renderHook(() => useRealmsApi());

      await expect(result.current.fetchActiveRealmIds()).rejects.toThrow('Network error');
    });
  });

  describe('updateActiveRealms', () => {
    it('updates active realms successfully', async () => {
      const realmIds = ['AstrumAuthenticatingRealm', 'LdapRealm'];
      mockedRestClient.put.mockResolvedValueOnce(null);

      const { result } = renderHook(() => useRealmsApi());

      await act(async () => {
        await result.current.updateActiveRealms(realmIds);
      });

      expect(mockedRestClient.put).toHaveBeenCalledWith('service/rest/v1/security/realms/active', realmIds);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('sets loading state during update', async () => {
      mockedRestClient.put.mockImplementation(() => new Promise((resolve) => setTimeout(() => resolve(null), 100)));

      const { result } = renderHook(() => useRealmsApi());

      let updatePromise: Promise<void>;
      act(() => {
        updatePromise = result.current.updateActiveRealms(['AstrumAuthenticatingRealm']);
      });

      expect(result.current.loading).toBe(true);

      await act(async () => {
        await updatePromise;
      });

      expect(result.current.loading).toBe(false);
    });

    it('sets error when update fails', async () => {
      const mockError = {
        response: { data: { message: 'Invalid realm configuration' } },
      };
      mockedRestClient.put.mockRejectedValueOnce(mockError);
      mockedParseApiError.mockReturnValueOnce({ message: 'Invalid realm configuration', status: 400 });

      const { result } = renderHook(() => useRealmsApi());

      await act(async () => {
        try {
          await result.current.updateActiveRealms(['InvalidRealm']);
        } catch (_e) {
          // Expected to throw
        }
      });

      expect(result.current.error).toBe('Invalid realm configuration');
    });

    it('uses default error message when no specific message provided', async () => {
      mockedRestClient.put.mockRejectedValueOnce(new Error());
      mockedParseApiError.mockReturnValueOnce({ message: 'An error occurred', status: undefined });

      const { result } = renderHook(() => useRealmsApi());

      await act(async () => {
        try {
          await result.current.updateActiveRealms(['AstrumAuthenticatingRealm']);
        } catch (_e) {
          // Expected to throw
        }
      });

      expect(result.current.error).toBe('An error occurred');
    });
  });

  describe('setError', () => {
    it('allows clearing the error', async () => {
      mockedRestClient.put.mockRejectedValueOnce(new Error('Test error'));
      mockedParseApiError.mockReturnValueOnce({ message: 'Test error', status: undefined });

      const { result } = renderHook(() => useRealmsApi());

      await act(async () => {
        try {
          await result.current.updateActiveRealms(['test']);
        } catch (_e) {
          // Expected
        }
      });

      expect(result.current.error).not.toBeNull();

      act(() => {
        result.current.setError(null);
      });

      expect(result.current.error).toBeNull();
    });
  });
});
