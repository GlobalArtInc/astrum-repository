

import { generate } from '../swift';
import type { SnippetComponentModel } from '../types';

describe('swift dependency snippet generator', () => {
  const component: SnippetComponentModel = {
    format: 'swift',
    group: '',
    name: 'apple.swift-collections',
    version: '1.1.0',
  };

  it('emits the Classic tool set in order', () => {
    expect(generate(component).map((s) => s.displayName)).toEqual(['Package.swift']);
  });

  it('produces snippet text matching Classic', () => {
    const [snippet] = generate(component);
    expect(snippet.description).toBe('Add the following dependency to your Package.swift:');
    expect(snippet.snippetText).toBe('.package(id: "apple.swift-collections", from: "1.1.0")');
  });
});
