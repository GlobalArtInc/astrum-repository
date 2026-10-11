

import type { SnippetGenerator } from './types';
import S from '../../../../constants/pages/search/ComponentDetailStrings';

/**
 * Cargo (Rust) dependency snippet generator.
 *
 * Ported from Classic AS.cargo.controller.CargoDependencySnippetController. Returns no snippets
 * for a versionless component, matching Classic.
 */
export const generate: SnippetGenerator = (component) => {
  const { name, version } = component;
  if (!version) {
    return [];
  }

  return [
    {
      displayName: 'cargo',
      description: S.SNIPPETS.RUN_IN_PROJECT,
      snippetText: `cargo add ${name}@${version}`,
    },
    {
      displayName: 'manual',
      description: S.SNIPPETS.ADD_LINE_TO('Cargo.toml'),
      snippetText: `${name} = "${version}"`,
    },
  ];
};
