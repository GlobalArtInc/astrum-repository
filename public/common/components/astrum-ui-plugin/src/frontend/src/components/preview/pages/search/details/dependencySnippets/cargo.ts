

import type { SnippetGenerator } from './types';

/**
 * Cargo (Rust) dependency snippet generator.
 *
 * Ported from Classic NX.cargo.controller.CargoDependencySnippetController. Returns no snippets
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
      description: 'Run the following command in your project directory:',
      snippetText: `cargo add ${name}@${version}`,
    },
    {
      displayName: 'manual',
      description: 'Add the following line to your Cargo.toml:',
      snippetText: `${name} = "${version}"`,
    },
  ];
};
