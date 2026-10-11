import { systemInfoMachine } from '../SystemInfoMachine';
import { APIConstants } from '../../../../../../../constants/APIConstants';

const mockGet = jest.fn();
jest.mock('../../../../../../../interface/api', () => ({
  restClient: { get: (...args: unknown[]) => mockGet(...args) },
  parseApiError: (e: Error) => ({ message: e.message }),
}));

let mockClustered = false;
jest.mock('../../../../../../../interface/ExtJS', () => ({
  ExtJS: { state: () => ({ getValue: (key: string) => key === 'astrum.datastore.clustered.enabled' && mockClustered }) },
}));

const fetchData = (systemInfoMachine.options.services as any).fetchData as () => Promise<any>;

describe('system information fetch', () => {
  beforeEach(() => {
    mockGet.mockReset();
    mockClustered = false;
  });

  it('does not ask a single node for the nodes of a cluster', async () => {
    mockGet.mockResolvedValue({ 'astrum-status': { version: '3.96.4' } });

    const result = await fetchData();

    expect(mockGet).toHaveBeenCalledTimes(1);
    expect(mockGet).toHaveBeenCalledWith(APIConstants.REST.SYSTEM_INFORMATION);
    expect(result.isHAMode).toBe(false);
    expect(result.nodes).toEqual([]);
  });

  it('reads the information of every node of a cluster', async () => {
    mockClustered = true;
    mockGet.mockImplementation((url: string) => {
      if (url === APIConstants.REST.INTERNAL.GET_SUPPORT_ZIP_ACTIVE_NODES) {
        return Promise.resolve([{ nodeId: 'node-1', local: true }, { nodeId: 'node-2', local: false }]);
      }
      return Promise.resolve({});
    });

    const result = await fetchData();

    expect(mockGet).toHaveBeenCalledWith(APIConstants.REST.INTERNAL.GET_SUPPORT_ZIP_ACTIVE_NODES);
    expect(mockGet).toHaveBeenCalledWith(APIConstants.REST.SYSTEM_INFORMATION_HA);
    expect(result.isHAMode).toBe(true);
    expect(result.selectedNode).toBe('node-1');
  });
});
