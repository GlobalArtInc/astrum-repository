

import type { SnippetGenerator } from './types';

/**
 * Terraform dependency snippet generator.
 *
 * Ported from Classic NX.terraform.controller.TerraformDependencySnippetController. Returns no
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
      description: 'Run the following command in your project directory:',
      snippetText: `terraform add ${name}@${version}`,
    },
    {
      displayName: 'manual',
      description: 'Add the following line to your Terraform.toml:',
      snippetText: `${name} = "${version}"`,
    },
  ];
};
