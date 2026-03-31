import { describe, expect, it } from 'vitest';
import { assetPath, treePath } from './panels';

describe('browse API paths', () => {
  it('keep the API prefix and encode the repository and path', () => {
    expect(treePath('raw files', '/a b/')).toBe(
      '/service/rest/internal/ui/browse/raw%20files?path=%2Fa+b%2F',
    );
    expect(assetPath('r', '/x.txt')).toBe('/service/rest/internal/ui/browse/r/asset?path=%2Fx.txt');
  });
});
