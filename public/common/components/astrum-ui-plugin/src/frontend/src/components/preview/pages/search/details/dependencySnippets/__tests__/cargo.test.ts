

import { generate } from '../cargo';
import type { SnippetComponentModel } from '../types';

describe('cargo dependency snippet generator', () => {
  const component: SnippetComponentModel = {
    format: 'cargo',
    group: '',
    name: 'serde',
    version: '1.0.197',
  };

  it('emits the Classic tool set in order', () => {
    expect(generate(component).map((s) => s.displayName)).toEqual(['cargo', 'manual']);
  });

  it('produces snippet text matching Classic', () => {
    const s: Record<string, string> = {};
    for (const g of generate(component)) s[g.displayName] = g.snippetText;
    expect(s['cargo']).toBe('cargo add serde@1.0.197');
    expect(s['manual']).toBe('serde = "1.0.197"');
  });

  it('returns no snippets when the version is missing', () => {
    expect(generate({ ...component, version: '' })).toEqual([]);
  });
});
