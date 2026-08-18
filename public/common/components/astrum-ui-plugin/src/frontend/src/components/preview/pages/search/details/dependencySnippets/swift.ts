

import type { SnippetGenerator } from './types';

/**
 * Swift dependency snippet generator.
 *
 * Ported from Classic NX.swift.controller.SwiftDependencySnippetController.
 */
export const generate: SnippetGenerator = (component) => {
  const { name, version } = component;

  return [
    {
      displayName: 'Package.swift',
      description: 'Add the following dependency to your Package.swift:',
      snippetText: `.package(id: "${name}", from: "${version}")`,
    },
  ];
};
