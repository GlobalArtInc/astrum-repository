

import { generate } from '../composer';
import type { SnippetComponentModel } from '../types';

describe('composer dependency snippet generator', () => {
  const component: SnippetComponentModel = {
    format: 'composer',
    group: 'symfony',
    name: 'console',
    version: '6.4.0',
  };

  it('emits the Classic tool set in order', () => {
    expect(generate(component).map((s) => s.displayName)).toEqual(['composer', 'manual']);
  });

  it('produces snippet text matching Classic', () => {
    const s: Record<string, string> = {};
    for (const g of generate(component)) s[g.displayName] = g.snippetText;
    expect(s['composer']).toBe('composer require symfony/console:6.4.0');
    expect(s['manual']).toBe('symfony/console: "6.4.0"');
  });

  it('returns no snippets when the version is missing', () => {
    expect(generate({ ...component, version: '' })).toEqual([]);
  });

  it('uses just the name when the component has no group (no leading slash)', () => {
    const s: Record<string, string> = {};
    for (const g of generate({ ...component, group: '' })) s[g.displayName] = g.snippetText;
    expect(s['composer']).toBe('composer require console:6.4.0');
    expect(s['manual']).toBe('console: "6.4.0"');
  });
});
