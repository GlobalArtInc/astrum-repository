

import type { SnippetGenerator } from './types';
import S from '../../../../constants/pages/search/ComponentDetailStrings';

/**
 * Terraform dependency snippet generator.
 *
 * The group of a component is its namespace; a module component is named `name/system` and a provider component
 * after its type. The registry host is left as a placeholder: the component view does not know which host the
 * repository answers for. Returns no snippets for a versionless component.
 */
export const generate: SnippetGenerator = (component) => {
  const { group, name, version } = component;
  if (!version || !name || !group) {
    return [];
  }
  const parts = name.split('/');
  if (parts.length === 2) {
    return [
      {
        displayName: 'module',
        description: S.SNIPPETS.ADD_LINE_TO('main.tf'),
        snippetText: `module "${parts[0]}" {\n  source  = "<registry host>/${group}/${name}"\n  version = "${version}"\n}`,
      },
    ];
  }
  if (parts.length === 1) {
    return [
      {
        displayName: 'required_providers',
        description: S.SNIPPETS.ADD_LINE_TO('main.tf'),
        snippetText:
          'terraform {\n  required_providers {\n' +
          `    ${name} = {\n      source  = "<registry host>/${group}/${name}"\n      version = "${version}"\n    }\n` +
          '  }\n}',
      },
    ];
  }
  return [];
};
