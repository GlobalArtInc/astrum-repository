

import type { SnippetGenerator } from './types';
import S from '../../../../constants/pages/search/ComponentDetailStrings';

/**
 * Swift dependency snippet generator.
 *
 * Ported from Classic AS.swift.controller.SwiftDependencySnippetController. A registry package is
 * identified as scope.name; the scope is the component's group.
 */
export const generate: SnippetGenerator = (component) => {
  const { group, name, version } = component;
  const identity = group ? `${group}.${name}` : name;

  return [
    {
      displayName: 'Package.swift',
      description: S.SNIPPETS.ADD_DEPENDENCY_TO('Package.swift'),
      snippetText: `.package(id: "${identity}", from: "${version}")`,
    },
  ];
};
