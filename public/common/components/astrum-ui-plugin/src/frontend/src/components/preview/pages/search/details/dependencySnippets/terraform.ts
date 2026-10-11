

import type { SnippetGenerator } from './types';
import S from '../../../../constants/pages/search/ComponentDetailStrings';

/**
 * Terraform dependency snippet generator.
 *
 * Ported from Classic AS.terraform.controller.TerraformDependencySnippetController. Returns no
 * snippets for a versionless component, matching Classic.
 */
export const generate: SnippetGenerator = (component) => {
  const { name, version } = component;
  if (!version) {
    return [];
  }

  return [
    {
      displayName: 'terraform',
      description: S.SNIPPETS.RUN_IN_PROJECT,
      snippetText: `terraform add ${name}@${version}`,
    },
    {
      displayName: 'manual',
      description: S.SNIPPETS.ADD_LINE_TO('Terraform.toml'),
      snippetText: `${name} = "${version}"`,
    },
  ];
};
