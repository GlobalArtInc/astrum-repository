

import type { SnippetGenerator } from './types';

/**
 * RubyGems dependency snippet generator.
 *
 * Ported from Classic AS.rubygems.controller.RubyGemsDependencySnippetController.
 */
export const generate: SnippetGenerator = (component) => {
  const { name, version } = component;

  return [
    { displayName: 'Install', snippetText: `gem install ${name} -v ${version}` },
    { displayName: 'Gemfile', snippetText: `gem '${name}', '~> ${version}'` },
  ];
};
