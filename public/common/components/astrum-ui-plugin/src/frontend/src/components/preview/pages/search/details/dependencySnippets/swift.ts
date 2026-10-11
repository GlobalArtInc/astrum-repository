

import type { SnippetGenerator } from './types';
import S from '../../../../constants/pages/search/ComponentDetailStrings';

/**
 * Swift dependency snippet generator.
 *
 * Ported from Classic AS.swift.controller.SwiftDependencySnippetController.
 */
export const generate: SnippetGenerator = (component) => {
  const { name, version } = component;

  return [
    {
      displayName: 'Package.swift',
      description: S.SNIPPETS.ADD_DEPENDENCY_TO('Package.swift'),
      snippetText: `.package(id: "${name}", from: "${version}")`,
    },
  ];
};
