

import { interpret } from 'xstate';
import { waitFor } from 'xstate/lib/waitFor';
import { createUsageMachine } from '../usageMachine';

jest.mock('../../../../../../../interface/api', () => ({
  restClient: { get: jest.fn() },
  parseApiError: (e: any) => ({
    message: e?.response?.data?.message ?? e?.message ?? 'err',
    status: e?.response?.status ?? e?.status ?? 0,
  }),
  isPermissionError: (apiErr: any) => apiErr?.status === 403,
}));
const { restClient } = jest.requireMock('../../../../../../../interface/api');

describe('usageMachine', () => {
  beforeEach(() => jest.clearAllMocks());

  it('loads monthly metrics and lands in loaded', async () => {
    restClient.get.mockResolvedValue([{ metricDate: '2026-01-01', componentCount: 5 }]);
    const service = interpret(createUsageMachine()).start();
    expect(service.getSnapshot().matches('loading')).toBe(true);
    await waitFor(service, (s) => s.matches('loaded'));
    expect(service.getSnapshot().context.metrics).toHaveLength(1);
    expect(service.getSnapshot().context.loadError).toBeNull();
    service.stop();
  });

  it('flags a 403 as a permission error', async () => {
    restClient.get.mockRejectedValue({ response: { status: 403 }, message: 'forbidden' });
    const service = interpret(createUsageMachine()).start();
    await waitFor(service, (s) => s.matches('loadError'));
    expect(service.getSnapshot().context.isPermissionError).toBe(true);
    service.stop();
  });

  it('sets loadError on a non-permission failure', async () => {
    restClient.get.mockRejectedValue({ message: 'network down' });
    const service = interpret(createUsageMachine()).start();
    await waitFor(service, (s) => s.matches('loadError'));
    expect(service.getSnapshot().context.isPermissionError).toBe(false);
    expect(service.getSnapshot().context.loadError).toBe('network down');
    service.stop();
  });

  it('RETRY from loadError re-fetches and reaches loaded', async () => {
    restClient.get.mockRejectedValueOnce({ message: 'x' }).mockResolvedValueOnce([]);
    const service = interpret(createUsageMachine()).start();
    await waitFor(service, (s) => s.matches('loadError'));
    service.send({ type: 'RETRY' });
    await waitFor(service, (s) => s.matches('loaded'));
    expect(service.getSnapshot().context.loadError).toBeNull();
    service.stop();
  });
});
