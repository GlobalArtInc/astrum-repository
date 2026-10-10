

import type { SnippetGenerator } from './types';

/**
 * PyPI dependency snippet generator.
 *
 * Ported from Classic AS.pypi.controller.PyPiDependencySnippetController.
 */
export const generate: SnippetGenerator = (component) => {
  const { name, version } = component;

  return [
    { displayName: 'pip', snippetText: `pip install ${name}==${version}` },
    { displayName: 'easy_install', snippetText: `easy_install ${name}==${version}` },
    { displayName: 'pipenv', snippetText: `pipenv install ${name}==${version}` },
    { displayName: 'requirements.txt', snippetText: `${name} == ${version}` },
  ];
};
