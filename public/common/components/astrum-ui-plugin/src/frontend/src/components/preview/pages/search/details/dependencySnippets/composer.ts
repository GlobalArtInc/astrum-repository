

import type { SnippetGenerator } from './types';

/**
 * Composer (PHP) dependency snippet generator.
 *
 * Ported from Classic AS.composer.controller.ComposerDependencySnippetController. Returns no
 * snippets for a versionless component, matching Classic.
 */
export const generate: SnippetGenerator = (component) => {
  const { group, name, version } = component;
  if (!version) {
    return [];
  }
  // Composer packages are normally vendor/package, but guard against a missing group so we
  // never emit a leading-slash name like "/console".
  const dependencyName = group ? `${group}/${name}` : name;

  return [
    {
      displayName: 'composer',
      description: 'Run the following command in your project directory:',
      snippetText: `composer require ${dependencyName}:${version}`,
    },
    {
      displayName: 'manual',
      description: 'Add the following line to your Composer.json:',
      snippetText: `${dependencyName}: "${version}"`,
    },
  ];
};
