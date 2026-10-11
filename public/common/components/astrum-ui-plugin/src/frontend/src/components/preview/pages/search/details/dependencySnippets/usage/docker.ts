import { generate as dockerSnippets } from '../docker';
import { sanitizeRegistryUrl } from '../../../../browse/detail/detail.utils';
import { usageStrings } from './strings';
import { hostOf, isHosted } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

const MANIFEST_PATH = /^\/?v2\/(.+)\/manifests\/([^/]+)$/;

/**
 * Image name and reference (tag or digest) of a Docker asset, read from its manifest path when there
 * is one: a manifest stored under a digest has no component, so its path is the only source.
 */
function imageOf(path: string | null | undefined, name: string | null | undefined, version: string | null | undefined) {
  const match = path ? MANIFEST_PATH.exec(path) : null;
  if (match) {
    let reference = match[2];
    try {
      reference = decodeURIComponent(reference);
    } catch {
      reference = match[2];
    }
    return { image: match[1], reference };
  }
  return { image: name ?? '', reference: version ?? '' };
}

/**
 * Docker usage: pull, sign in, Dockerfile, Compose, Kubernetes and, for a hosted repository, push.
 *
 * The registry is the one the server computed for the repository; without it the repository is
 * addressed by path on the server's own host, which is how it is reached when no connector is set.
 */
export const generate: UsageGenerator = (context) => {
  const { image, reference } = imageOf(context.path, context.name, context.version);
  if (!image || !reference) {
    return [];
  }
  const registry = sanitizeRegistryUrl(context.registryUrl) ??
    `${hostOf(context.repositoryUrl)}/${context.repositoryName}`;
  const loginHost = registry.split('/')[0];
  const isDigest = reference.startsWith('sha256:');
  const imageRef = `${registry}/${image}${isDigest ? '@' : ':'}${reference}`;
  const strings = usageStrings();

  const classic = dockerSnippets(
    { format: 'docker', group: '', name: image, version: reference },
    { registryUrl: registry },
  );
  const byName = (displayName: string) =>
    (classic.find((snippet) => snippet.displayName === displayName)?.snippetText ?? '')
      .split(`${registry}/${image}:${reference}`).join(imageRef);

  const snippets: UsageSnippet[] = [
    { id: 'pull', label: 'docker pull', snippetText: `docker pull ${imageRef}` },
    {
      id: 'login',
      label: 'docker login',
      snippetText: `docker login ${loginHost}`,
      description: strings.HINTS.DOCKER_LOGIN,
    },
    {
      id: 'dockerfile',
      label: 'Dockerfile',
      snippetText: byName('Dockerfile'),
      description: strings.HINTS.DOCKERFILE,
    },
    { id: 'compose', label: 'Compose', snippetText: byName('Compose') },
    { id: 'kubernetes', label: 'Kubernetes', snippetText: byName('Kubernetes') },
  ];
  if (isHosted(context.repositoryType) && !isDigest) {
    snippets.push({
      id: 'push',
      label: 'docker push',
      snippetText: `docker tag ${image}:${reference} ${imageRef}\ndocker push ${imageRef}`,
      description: strings.HINTS.DOCKER_PUSH,
    });
  }
  return snippets;
};
