

import { generate } from '../terraform';
import type { SnippetComponentModel } from '../types';

describe('terraform dependency snippet generator', () => {
  const component: SnippetComponentModel = {
    format: 'terraform',
    group: '',
    name: 'hashicorp/aws',
    version: '5.40.0',
  };

  it('emits the Classic tool set in order', () => {
    expect(generate(component).map((s) => s.displayName)).toEqual(['terraform', 'manual']);
  });

  it('produces snippet text matching Classic', () => {
    const s: Record<string, string> = {};
    for (const g of generate(component)) s[g.displayName] = g.snippetText;
    expect(s['terraform']).toBe('terraform add hashicorp/aws@5.40.0');
    expect(s['manual']).toBe('hashicorp/aws = "5.40.0"');
  });

  it('returns no snippets when the version is missing', () => {
    expect(generate({ ...component, version: '' })).toEqual([]);
  });
});
