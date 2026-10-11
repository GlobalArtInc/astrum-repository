

import { renderHook, act, waitFor } from '@testing-library/react';

import type { ApiPermissionsResponseDto } from '../../types';

jest.mock('../../../../../../../../interface/api', () => ({
  ...jest.requireActual('../../../../../../../../interface/api'),
  restClient: {
    get: jest.fn(),
    post: jest.fn(),
    put: jest.fn(),
    delete: jest.fn(),
  },
}));

import { restClient } from '../../../../../../../../interface/api';
import { useEndpointPermissions } from '../useEndpointPermissions';

const mockGet = restClient.get as jest.MockedFunction<typeof restClient.get>;

const MOCK_RESPONSE: ApiPermissionsResponseDto = {
  endpoints: [
    {
      httpMethod: 'GET',
      pathPattern: '/service/rest/v1/status',
      permissions: [],
      description: 'Health check',
      tag: 'Status',
      authenticated: false,
    },
  ],
  generatedAt: '2026-03-24T00:00:00Z',
  totalEndpoints: 1,
  unmappedEndpoints: 0,
};

describe('useEndpointPermissions', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('fetches permissions on mount and returns data', async () => {
    mockGet.mockResolvedValueOnce(MOCK_RESPONSE);

    const { result } = renderHook(() => useEndpointPermissions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.data).toEqual(MOCK_RESPONSE);
    expect(result.current.error).toBeNull();
    expect(mockGet).toHaveBeenCalledWith('/service/rest/internal/ui/api/permissions');
  });

  it('uses session cache on subsequent mounts without refetching', () => {
    const { result } = renderHook(() => useEndpointPermissions());

    expect(result.current.loading).toBe(false);
    expect(result.current.data).toEqual(MOCK_RESPONSE);
    expect(mockGet).not.toHaveBeenCalled();
  });

  it('exposes a refetch function that reloads data', async () => {
    const { result } = renderHook(() => useEndpointPermissions());
    await waitFor(() => expect(result.current.loading).toBe(false));

    const updated: ApiPermissionsResponseDto = { ...MOCK_RESPONSE, totalEndpoints: 5 };
    mockGet.mockResolvedValueOnce(updated);

    await act(async () => {
      await result.current.refetch();
    });

    expect(result.current.data).toEqual(updated);
  });

  it('sets error state on fetch failure via refetch', async () => {
    const { result } = renderHook(() => useEndpointPermissions());
    await waitFor(() => expect(result.current.loading).toBe(false));

    mockGet.mockRejectedValueOnce(new Error('Network error'));

    await act(async () => {
      await result.current.refetch();
    });

    expect(result.current.data).toBeNull();
    expect(result.current.error).toBe('Network error');
  });

  it('uses fallback error message for non-Error throws', async () => {
    const { result } = renderHook(() => useEndpointPermissions());
    await waitFor(() => expect(result.current.loading).toBe(false));

    mockGet.mockRejectedValueOnce('something');

    await act(async () => {
      await result.current.refetch();
    });

    expect(result.current.error).toBe('Failed to load API permissions');
  });
});
