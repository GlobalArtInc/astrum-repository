

import { generate } from '../terraform';
import type { SnippetComponentModel } from '../types';

describe('terraform dependency snippet generator', () => {
  const provider: SnippetComponentModel = {
    format: 'terraform',
    group: 'hashicorp',
    name: 'aws',
    version: '5.40.0',
  };

  it('declares a provider requirement for a provider component', () => {
    const snippets = generate(provider);
    expect(snippets.map((s) => s.displayName)).toEqual(['required_providers']);
    expect(snippets[0].snippetText).toBe(
      'terraform {\n  required_providers {\n    aws = {\n      source  = "<registry host>/hashicorp/aws"\n' +
        '      version = "5.40.0"\n    }\n  }\n}',
    );
  });

  it('declares a module block for a module component', () => {
    const snippets = generate({ ...provider, group: 'acme', name: 'vpc/aws', version: '1.2.0' });
    expect(snippets.map((s) => s.displayName)).toEqual(['module']);
    expect(snippets[0].snippetText).toBe(
      'module "vpc" {\n  source  = "<registry host>/acme/vpc/aws"\n  version = "1.2.0"\n}',
    );
  });

  it('returns no snippets when the version is missing or the name is no address', () => {
    expect(generate({ ...provider, version: '' })).toEqual([]);
    expect(generate({ ...provider, group: '' })).toEqual([]);
    expect(generate({ ...provider, name: 'a/b/c' })).toEqual([]);
  });
});
