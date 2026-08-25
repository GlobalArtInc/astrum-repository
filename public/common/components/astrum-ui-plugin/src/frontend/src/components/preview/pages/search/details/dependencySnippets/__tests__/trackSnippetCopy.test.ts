

jest.mock('axios', () => ({
  post: jest.fn(),
}));

import Axios from 'axios';
import { trackSnippetCopy } from '../trackSnippetCopy';

const mockAxios = Axios as jest.Mocked<typeof Axios>;

describe('trackSnippetCopy', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockAxios.post.mockResolvedValue({} as never);
  });

  it('POSTs to the dependency_snippets endpoint with the normalized format and snippet name', () => {
    trackSnippetCopy('maven', 'Apache Maven');
    expect(mockAxios.post).toHaveBeenCalledTimes(1);
    expect(mockAxios.post).toHaveBeenCalledWith(
      '/service/rest/dependency_snippets?format=maven2&snippet=Apache%20Maven',
    );
  });

  it('URL-encodes spaces in the snippet display name', () => {
    trackSnippetCopy('oci', 'OCI Pull (ORAS)');
    expect(mockAxios.post).toHaveBeenCalledWith(
      '/service/rest/dependency_snippets?format=oci&snippet=OCI%20Pull%20(ORAS)',
    );
  });

  it('does not throw when the request rejects (fire-and-forget)', () => {
    mockAxios.post.mockRejectedValue(new Error('network'));
    expect(() => trackSnippetCopy('npm', 'npm')).not.toThrow();
  });
});
