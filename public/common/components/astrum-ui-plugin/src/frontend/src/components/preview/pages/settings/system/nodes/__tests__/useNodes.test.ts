

import { act, renderHook, waitFor } from '@testing-library/react';

import { useNodes } from '../useNodes';

jest.mock('../nodesApi', () => ({ fetchNodes: jest.fn() }));
const { fetchNodes } = jest.requireMock('../nodesApi');

const NODES = [{ name: 'uuid-1', displayName: 'Primary', local: true }];

describe('useNodes', () => {
  beforeEach(() => jest.clearAllMocks());

  it('exposes loading then loaded nodes', async () => {
    fetchNodes.mockResolvedValue(NODES);
    const { result } = renderHook(() => useNodes());

    expect(result.current.loading).toBe(true);
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.nodes).toEqual(NODES);
    expect(result.current.error).toBeNull();
  });

  it('exposes the error message on failure', async () => {
    fetchNodes.mockRejectedValue(new Error('boom'));
    const { result } = renderHook(() => useNodes());

    await waitFor(() => expect(result.current.error).toBe('boom'));
    expect(result.current.loading).toBe(false);
  });

  it('refresh triggers a re-fetch', async () => {
    fetchNodes.mockResolvedValue(NODES);
    const { result } = renderHook(() => useNodes());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(fetchNodes).toHaveBeenCalledTimes(1);

    act(() => result.current.refresh());
    await waitFor(() => expect(fetchNodes).toHaveBeenCalledTimes(2));
  });

  it('retry clears the error and re-fetches', async () => {
    fetchNodes.mockRejectedValueOnce(new Error('boom'));
    fetchNodes.mockResolvedValueOnce(NODES);
    const { result } = renderHook(() => useNodes());
    await waitFor(() => expect(result.current.error).toBe('boom'));

    act(() => result.current.retry());
    await waitFor(() => expect(result.current.error).toBeNull());
    expect(result.current.nodes).toEqual(NODES);
  });
});
