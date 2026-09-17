

import { renderHook, act, waitFor } from '@testing-library/react';
import { useOAuth2Api } from '../useOAuth2Api';

const MOCK_OAUTH2_URL = 'service/rest/internal/ui/oauth2';

const mockRestClient = {
  get: jest.fn(),
  post: jest.fn(),
  put: jest.fn(),
  delete: jest.fn(),
};

// Mock the REST API from @/utils/api
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

describe('useOAuth2Api', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('fetches OAuth2 config and stringifies JSON fields', async () => {
    const apiConfig = {
      clientId: 'client-id',
      clientSecret: 'secret',
      idpAuthorizationUrl: 'https://auth.example.com/authorize',
      idpLogoutUrl: 'https://auth.example.com/logout',
      idpTokenUrl: 'https://auth.example.com/token',
      idpJwksUrl: 'https://auth.example.com/jwks',
      idpJwsAlgorithm: 'RS256',
      usernameClaim: 'sub',
      exactMatchClaims: { groups: ['admins'] },
      authorizationCustomParams: { prompt: 'login' },
      tokenRequestCustomParams: { audience: 'astrum' },
    };

    mockRestClient.get.mockResolvedValue(apiConfig);

    const { result } = renderHook(() => useOAuth2Api());

    let config;
    await act(async () => {
      config = await result.current.fetchConfig();
    });

    expect(config).toEqual(
      expect.objectContaining({
        clientId: 'client-id',
        idpTokenUrl: 'https://auth.example.com/token',
        exactMatchClaims: JSON.stringify({ groups: ['admins'] }, null, 2),
        authorizationCustomParams: JSON.stringify({ prompt: 'login' }, null, 2),
        tokenRequestCustomParams: JSON.stringify({ audience: 'astrum' }, null, 2),
      })
    );
    expect(mockRestClient.get).toHaveBeenCalledWith(MOCK_OAUTH2_URL);
  });

  it('throws error with parsed message when fetch fails', async () => {
    const errorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});
    mockRestClient.get.mockRejectedValue({ response: { data: { message: 'Not authorized' } } });

    const { result } = renderHook(() => useOAuth2Api());

    await expect(result.current.fetchConfig()).rejects.toThrow('Not authorized');
    errorSpy.mockRestore();
  });

  it('saves OAuth2 config with parsed JSON fields', async () => {
    const configToSave = {
      clientId: 'client-id',
      clientSecret: 'secret',
      idpAuthorizationUrl: 'https://auth.example.com/authorize',
      idpLogoutUrl: '',
      idpTokenUrl: 'https://auth.example.com/token',
      idpJwksUrl: 'https://auth.example.com/jwks',
      idpJwsAlgorithm: 'RS256',
      usernameClaim: 'sub',
      firstNameClaim: '',
      lastNameClaim: '',
      emailClaim: '',
      groupsClaim: '',
      exactMatchClaims: JSON.stringify({ groups: ['admins'] }),
      authorizationCustomParams: JSON.stringify({ prompt: 'login' }),
      tokenRequestCustomParams: JSON.stringify({ audience: 'astrum' }),
    };

    mockRestClient.put.mockResolvedValue({});

    const { result } = renderHook(() => useOAuth2Api());

    await act(async () => {
      await result.current.saveConfig(configToSave);
    });

    expect(mockRestClient.put).toHaveBeenCalledWith(
      MOCK_OAUTH2_URL,
      expect.objectContaining({
        clientId: 'client-id',
        exactMatchClaims: { groups: ['admins'] },
        authorizationCustomParams: { prompt: 'login' },
        tokenRequestCustomParams: { audience: 'astrum' },
      })
    );
  });

  it('sets loading state during save', async () => {
    let resolvePromise: (value: unknown) => void;
    const pendingPromise = new Promise((resolve) => {
      resolvePromise = resolve;
    });
    mockRestClient.put.mockReturnValueOnce(pendingPromise as any);

    const { result } = renderHook(() => useOAuth2Api());

    expect(result.current.loading).toBe(false);

    let savePromise: Promise<void>;
    await act(async () => {
      savePromise = result.current.saveConfig({
        clientId: 'client-id',
        clientSecret: 'secret',
        idpAuthorizationUrl: '',
        idpLogoutUrl: '',
        idpTokenUrl: '',
        idpJwksUrl: '',
        idpJwsAlgorithm: '',
        usernameClaim: '',
        firstNameClaim: '',
        lastNameClaim: '',
        emailClaim: '',
        groupsClaim: '',
        exactMatchClaims: '',
        authorizationCustomParams: '',
        tokenRequestCustomParams: '',
      });
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(true);
    });

    await act(async () => {
      resolvePromise!({});
      await savePromise!;
    });

    expect(result.current.loading).toBe(false);
  });

  it('sets error state on save failure', async () => {
    mockRestClient.put.mockRejectedValue(new Error('Save failed'));

    const { result } = renderHook(() => useOAuth2Api());

    await act(async () => {
      await expect(
        result.current.saveConfig({
          clientId: '',
          clientSecret: '',
          idpAuthorizationUrl: '',
          idpLogoutUrl: '',
          idpTokenUrl: '',
          idpJwksUrl: '',
          idpJwsAlgorithm: '',
          usernameClaim: '',
          firstNameClaim: '',
          lastNameClaim: '',
          emailClaim: '',
          groupsClaim: '',
          exactMatchClaims: '',
          authorizationCustomParams: '',
          tokenRequestCustomParams: '',
        })
      ).rejects.toThrow('Save failed');
    });

    await waitFor(() => {
      expect(result.current.error).toBe('Save failed');
    });
  });

  it('clears error with setError', () => {
    const { result } = renderHook(() => useOAuth2Api());

    act(() => {
      result.current.setError('Test error');
    });

    act(() => {
      result.current.setError(null);
    });

    expect(result.current.error).toBeNull();
  });
});

