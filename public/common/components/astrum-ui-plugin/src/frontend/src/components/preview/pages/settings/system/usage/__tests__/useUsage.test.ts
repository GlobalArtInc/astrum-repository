
import { renderHook, act, waitFor } from '@testing-library/react';
import { useUsage } from '../useUsage';

jest.mock('../../../../../../../interface/api', () => ({
  restClient: { get: jest.fn() },
  parseApiError: (e: any) => ({
    message: e?.response?.data?.message ?? e?.message ?? 'err',
    status: e?.response?.status ?? e?.status ?? 0,
  }),
  isPermissionError: (apiErr: any) => apiErr?.status === 403,
}));
const { restClient } = jest.requireMock('../../../../../../../interface/api');

describe('useUsage', () => {
  beforeEach(() => jest.clearAllMocks());

  it('loads monthly metrics and the daily chart on mount', async () => {
    restClient.get.mockImplementation((url: string) => {
      if (url.includes('monthly-metrics')) {
        return Promise.resolve([{ metricDate: '2026-01-01', egress: 5, storage: 6 }]);
      }
      return Promise.resolve({ data: [] });
    });
    const { result } = renderHook(() => useUsage());
    await waitFor(() => expect(result.current.loading).toBe(false));
    await waitFor(() => expect(result.current.chartLoading).toBe(false));
    expect(result.current.metrics).toHaveLength(1);
    expect(restClient.get).toHaveBeenCalledWith('service/rest/v1/monthly-metrics');
    expect(restClient.get).toHaveBeenCalledWith('service/rest/v1/daily-metrics/egress', expect.anything());
    expect(restClient.get).toHaveBeenCalledWith('service/rest/v1/daily-metrics/storage', expect.anything());
    // Assert the hook's derived chart output, not just that the calls were made.
    expect(result.current.chartError).toBeNull();
    expect(result.current.chartData).toEqual([]);
    expect(result.current.monthOptions.length).toBeGreaterThan(0);
    expect(result.current.selectedMonth).not.toBeNull();
  });

  it('shows the storage note by default and hides it on dismiss', async () => {
    restClient.get.mockResolvedValue([]);
    const { result } = renderHook(() => useUsage());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.storageNoteVisible).toBe(true);
    act(() => result.current.dismissStorageNote());
    expect(result.current.storageNoteVisible).toBe(false);
  });
});
