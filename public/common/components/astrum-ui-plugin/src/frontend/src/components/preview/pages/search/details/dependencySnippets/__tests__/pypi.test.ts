

import { generate } from '../pypi';
import type { SnippetComponentModel } from '../types';

describe('pypi dependency snippet generator', () => {
  const component: SnippetComponentModel = {
    format: 'pypi',
    group: '',
    name: 'requests',
    version: '2.31.0',
  };

  it('emits the Classic tool set in order', () => {
    expect(generate(component).map((s) => s.displayName)).toEqual([
      'pip',
      'easy_install',
      'pipenv',
      'requirements.txt',
    ]);
  });

  it('produces snippet text matching Classic', () => {
    const s: Record<string, string> = {};
    for (const g of generate(component)) s[g.displayName] = g.snippetText;
    expect(s['pip']).toBe('pip install requests==2.31.0');
    expect(s['easy_install']).toBe('easy_install requests==2.31.0');
    expect(s['pipenv']).toBe('pipenv install requests==2.31.0');
    expect(s['requirements.txt']).toBe('requests == 2.31.0');
  });
});
