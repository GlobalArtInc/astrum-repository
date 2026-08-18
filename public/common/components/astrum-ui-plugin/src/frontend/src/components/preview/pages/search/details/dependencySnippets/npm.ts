

import type { SnippetGenerator } from './types';

/**
 * npm dependency snippet generator.
 *
 * Ported from Classic NX.npm.controller.NpmDependencySnippetController. A non-empty group is
 * treated as the package scope and rendered as `@scope/name`.
 */
export const generate: SnippetGenerator = (component) => {
  const { group, name, version } = component;
  const dependencyName = (group ? `@${group}/` : '') + name;

  return [
    {
      displayName: 'npm',
      description: 'Install runtime dependency',
      snippetText: `npm install ${dependencyName}@${version}`,
    },
    {
      displayName: 'Yarn',
      description: 'Install runtime dependency',
      snippetText: `yarn add ${dependencyName}@${version}`,
    },
    {
      displayName: 'package.json',
      description: 'Install runtime dependency to the package.json\'s "dependencies" section',
      snippetText: `"${dependencyName}": "${version}"`,
    },
  ];
};
