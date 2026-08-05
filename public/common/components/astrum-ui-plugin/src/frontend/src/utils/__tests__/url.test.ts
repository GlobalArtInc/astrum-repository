

import { ensureTrailingSlash } from '../url';

describe('ensureTrailingSlash', () => {
  it('appends a trailing slash when the URL does not end with one', () => {
    expect(ensureTrailingSlash('http://example.com')).toBe('http://example.com/');
  });

  it('returns the URL unchanged when it already ends with a slash', () => {
    expect(ensureTrailingSlash('http://example.com/')).toBe('http://example.com/');
  });

  it('handles a plain path without trailing slash', () => {
    expect(ensureTrailingSlash('/api/v1')).toBe('/api/v1/');
  });

  it('handles a plain path that already has a trailing slash', () => {
    expect(ensureTrailingSlash('/api/v1/')).toBe('/api/v1/');
  });

  it('handles an empty string by appending a slash', () => {
    expect(ensureTrailingSlash('')).toBe('/');
  });
});
