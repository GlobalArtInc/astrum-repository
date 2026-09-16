

import { fetchCrowdConfig, saveCrowdConfig, verifyCrowdConnection, clearCrowdCache } from '../crowdApi';

const CROWD_URL = '/service/rest/v1/security/atlassian-crowd';

const mockRestClient = {
  get: jest.fn(),
  post: jest.fn(),
  put: jest.fn(),
  delete: jest.fn(),
};

jest.mock('../../../../../../../interface/api', () => ({
  ...jest.requireActual('../../../../../../../interface/api'),
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

const CONFIG = {
  enabled: true,
  realmActive: false,
  url: 'http://crowd.example.com',
  useTrustStoreForUrl: false,
  applicationName: 'astrum',
  applicationPassword: 'secret',
  timeout: 30,
};

describe('crowdApi', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('fetches config merged over defaults', async () => {
    mockRestClient.get.mockResolvedValue({ url: 'http://crowd.example.com', applicationName: 'astrum' });
    const config = await fetchCrowdConfig();
    expect(config).toEqual(
      expect.objectContaining({ url: 'http://crowd.example.com', applicationName: 'astrum', enabled: false })
    );
    expect(mockRestClient.get).toHaveBeenCalledWith(CROWD_URL);
  });

  it('throws parsed message on fetch failure', async () => {
    const errorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});
    mockRestClient.get.mockRejectedValue({ response: { data: { message: 'Forbidden' } } });
    await expect(fetchCrowdConfig()).rejects.toThrow('Forbidden');
    errorSpy.mockRestore();
  });

  it('saves config via PUT', async () => {
    mockRestClient.put.mockResolvedValue({});
    await saveCrowdConfig(CONFIG);
    expect(mockRestClient.put).toHaveBeenCalledWith(CROWD_URL, CONFIG);
  });

  it('throws parsed message on save failure', async () => {
    mockRestClient.put.mockRejectedValue(new Error('Save failed'));
    await expect(saveCrowdConfig(CONFIG)).rejects.toThrow('Save failed');
  });

  it('verifies connection via POST', async () => {
    mockRestClient.post.mockResolvedValue({});
    await verifyCrowdConnection(CONFIG);
    expect(mockRestClient.post).toHaveBeenCalledWith(`${CROWD_URL}/verify-connection`, CONFIG);
  });

  it('clears cache via POST', async () => {
    mockRestClient.post.mockResolvedValue({});
    await clearCrowdCache();
    expect(mockRestClient.post).toHaveBeenCalledWith(`${CROWD_URL}/clear-cache`);
  });
});
