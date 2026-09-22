

import { interpret } from 'xstate';
import { waitFor } from 'xstate/lib/waitFor';
import { createNodesMachine } from '../nodesMachine';

jest.mock('../nodesApi', () => ({ fetchNodes: jest.fn() }));
const { fetchNodes } = jest.requireMock('../nodesApi');

const NODES = [
  { name: 'uuid-1', displayName: 'Primary', local: true },
  { name: 'uuid-2', displayName: 'Secondary', local: false },
];

describe('nodesMachine', () => {
  beforeEach(() => jest.clearAllMocks());

  it('starts in loading then transitions to loaded with nodes', async () => {
    fetchNodes.mockResolvedValue(NODES);
    const service = interpret(createNodesMachine()).start();

    expect(service.getSnapshot().matches('loading')).toBe(true);

    await waitFor(service, (s) => s.matches('loaded'));
    expect(service.getSnapshot().context.nodes).toEqual(NODES);
    expect(service.getSnapshot().context.loadError).toBeNull();

    service.stop();
  });

  it('transitions to error with the failure message', async () => {
    fetchNodes.mockRejectedValue(new Error('Network error'));
    const service = interpret(createNodesMachine()).start();

    await waitFor(service, (s) => s.matches('error'));
    expect(service.getSnapshot().context.loadError).toBe('Network error');

    service.stop();
  });

  it('retries from error on RETRY', async () => {
    fetchNodes.mockRejectedValueOnce(new Error('Network error'));
    fetchNodes.mockResolvedValueOnce(NODES);
    const service = interpret(createNodesMachine()).start();

    await waitFor(service, (s) => s.matches('error'));
    service.send({ type: 'RETRY' });

    await waitFor(service, (s) => s.matches('loaded'));
    expect(service.getSnapshot().context.nodes).toEqual(NODES);
    expect(service.getSnapshot().context.loadError).toBeNull();

    service.stop();
  });

  it('re-fetches from loaded on REFRESH', async () => {
    fetchNodes.mockResolvedValue(NODES);
    const service = interpret(createNodesMachine()).start();

    await waitFor(service, (s) => s.matches('loaded'));
    service.send({ type: 'REFRESH' });
    expect(service.getSnapshot().matches('loading')).toBe(true);

    await waitFor(service, (s) => s.matches('loaded'));
    expect(fetchNodes).toHaveBeenCalledTimes(2);

    service.stop();
  });

  it('clears stale nodes when a refresh fails', async () => {
    fetchNodes.mockResolvedValueOnce(NODES);
    fetchNodes.mockRejectedValueOnce(new Error('Network error'));
    const service = interpret(createNodesMachine()).start();

    await waitFor(service, (s) => s.matches('loaded'));
    expect(service.getSnapshot().context.nodes).toEqual(NODES);

    service.send({ type: 'REFRESH' });

    await waitFor(service, (s) => s.matches('error'));
    expect(service.getSnapshot().context.nodes).toEqual([]);
    expect(service.getSnapshot().context.loadError).toBe('Network error');

    service.stop();
  });
});
