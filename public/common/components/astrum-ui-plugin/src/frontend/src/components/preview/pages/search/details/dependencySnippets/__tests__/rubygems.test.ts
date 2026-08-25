

import { generate } from '../rubygems';
import type { SnippetComponentModel } from '../types';

describe('rubygems dependency snippet generator', () => {
  const component: SnippetComponentModel = {
    format: 'rubygems',
    group: '',
    name: 'rails',
    version: '7.1.3',
  };

  it('emits the Classic tool set in order', () => {
    expect(generate(component).map((s) => s.displayName)).toEqual(['Install', 'Gemfile']);
  });

  it('produces snippet text matching Classic', () => {
    const s: Record<string, string> = {};
    for (const g of generate(component)) s[g.displayName] = g.snippetText;
    expect(s['Install']).toBe('gem install rails -v 7.1.3');
    expect(s['Gemfile']).toBe("gem 'rails', '~> 7.1.3'");
  });
});
