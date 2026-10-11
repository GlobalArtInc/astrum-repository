import { renderHook, waitFor } from '@testing-library/react';
import { useRepositoryList, resetHealthCheckAvailability } from '../useRepositoryList';

jest.mock('../../../../../../interface/ExtJS', () => ({
  ExtJS: {
    state: jest.fn(() => ({ getValue: jest.fn(() => undefined) })),
    checkPermission: jest.fn(() => true),
  },
}));

const mockGet = jest.fn();

jest.mock('../../../../../../interface/api', () => {
  const actual = jest.requireActual('../../../../../../interface/api');
  return {
    ...actual,
    restClient: {
      get: (...args: unknown[]) => mockGet(...args),
      post: jest.fn(),
    },
  };
});

function notFound() {
  return { isAxiosError: true, message: 'Not Found', response: { status: 404, data: {}, headers: {} } };
}

function answer(healthCheck: () => Promise<unknown>) {
  mockGet.mockImplementation((url: string) => {
    if (url.includes('healthcheck')) {
      return healthCheck();
    }
    return Promise.resolve([]);
  });
}

describe('useRepositoryList health check', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    resetHealthCheckAvailability();
  });

  it('shows the health check column once the server reports the statuses', async () => {
    answer(() => Promise.resolve([{ repositoryName: 'maven-central', enabled: true }]));

    const { result } = renderHook(() => useRepositoryList());

    await waitFor(() => expect(result.current.showHealthCheckColumn).toBe(true));
    expect(result.current.state.healthCheck['maven-central'].enabled).toBe(true);
  });

  it('hides the health check column when the server has no health check API', async () => {
    answer(() => Promise.reject(notFound()));

    const { result } = renderHook(() => useRepositoryList());

    await waitFor(() => expect(result.current.state.healthCheckAvailable).toBe(false));
    expect(result.current.showHealthCheckColumn).toBe(false);
    expect(result.current.state.healthCheckError).toBeUndefined();
  });

  it('does not ask again once the server answered that it has no health check API', async () => {
    answer(() => Promise.reject(notFound()));
    const first = renderHook(() => useRepositoryList());
    await waitFor(() => expect(first.result.current.state.healthCheckAvailable).toBe(false));
    first.unmount();
    mockGet.mockClear();

    const { result } = renderHook(() => useRepositoryList());

    await waitFor(() => expect(result.current.state.healthCheckAvailable).toBe(false));
    expect(mockGet.mock.calls.some(([url]) => String(url).includes('healthcheck'))).toBe(false);
  });

  it('keeps the column with an error when reading the statuses fails otherwise', async () => {
    answer(() => Promise.reject(new Error('boom')));

    const { result } = renderHook(() => useRepositoryList());

    await waitFor(() => expect(result.current.state.healthCheckError).toBeDefined());
    expect(result.current.showHealthCheckColumn).toBe(true);
  });
});
