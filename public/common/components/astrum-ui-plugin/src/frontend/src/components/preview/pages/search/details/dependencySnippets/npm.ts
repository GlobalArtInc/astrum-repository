

import type { SnippetGenerator } from './types';
import S from '../../../../constants/pages/search/ComponentDetailStrings';

/**
 * npm dependency snippet generator.
 *
 * Ported from Classic AS.npm.controller.NpmDependencySnippetController. A non-empty group is
 * treated as the package scope and rendered as `@scope/name`.
 */
export const generate: SnippetGenerator = (component) => {
  const { group, name, version } = component;
  const dependencyName = (group ? `@${group}/` : '') + name;

  return [
    {
      displayName: 'npm',
      description: S.SNIPPETS.NPM_INSTALL,
      snippetText: `npm install ${dependencyName}@${version}`,
    },
    {
      displayName: 'Yarn',
      description: S.SNIPPETS.NPM_INSTALL,
      snippetText: `yarn add ${dependencyName}@${version}`,
    },
    {
      displayName: 'package.json',
      description: S.SNIPPETS.NPM_PACKAGE_JSON,
      snippetText: `"${dependencyName}": "${version}"`,
    },
  ];
};
