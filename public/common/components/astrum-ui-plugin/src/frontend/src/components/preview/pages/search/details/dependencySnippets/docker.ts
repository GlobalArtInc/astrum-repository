

import type { SnippetGenerator } from './types';

/**
 * Docker dependency snippet generator.
 *
 * Ported from Classic AS.docker.controller.DockerDependencySnippetController. When the asset
 * carries a top-level registry URL (ASTRUM-51972) the pull target is prefixed with it.
 */
export const generate: SnippetGenerator = (component, asset) => {
  const yamlIndent = '  ';
  const { name, version } = component;
  const shortName = name.substring(name.lastIndexOf('/') + 1);
  const registry = asset?.registryUrl;
  const pullTarget = registry ? `${registry}/${name}` : name;

  // Every reference to the image must use the registry-qualified target; a bare name would
  // resolve to Docker Hub instead of this Astrum registry when a registryUrl is present.
  return [
    { displayName: 'Docker', snippetText: `docker pull ${pullTarget}:${version}` },
    { displayName: 'Dockerfile', snippetText: `FROM ${pullTarget}:${version}` },
    {
      displayName: 'Compose',
      snippetText:
        'services:\n' +
        `${yamlIndent}${shortName}:\n` +
        `${yamlIndent.repeat(2)}image: ${pullTarget}:${version}`,
    },
    {
      displayName: 'Kubernetes',
      snippetText:
        'spec:\n' +
        `${yamlIndent}containers:\n` +
        `${yamlIndent}- name: ${shortName}\n` +
        `${yamlIndent.repeat(2)}image: ${pullTarget}:${version}`,
    },
  ];
};
