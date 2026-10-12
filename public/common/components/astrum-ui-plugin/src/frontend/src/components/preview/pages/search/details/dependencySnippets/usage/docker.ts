import { generate as dockerSnippets } from '../docker';
import { sanitizeRegistryUrl } from '../../../../browse/detail/detail.utils';
import { dockerArtifactKind, dockerAttributesOf } from '../../../../../shared/DockerArtifact';
import { usageStrings } from './strings';
import { hostOf, isHosted, isPlainHttp } from './urls';
import type { UsageContext, UsageGenerator, UsageSnippet } from './types';

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

interface Target {
  /** host[:port][/repository] the clients address */
  registry: string;
  /** host[:port] the clients sign in to */
  loginHost: string;
  image: string;
  reference: string;
  isDigest: boolean;
  /** registry/image:tag or registry/image@digest */
  ref: string;
  /** ` --plain-http` when the server is reached over plain HTTP, which oras and helm refuse otherwise */
  plainHttp: string;
}

function targetOf(context: UsageContext): Target | null {
  const { image, reference } = imageOf(context.path, context.name, context.version);
  if (!image || !reference) {
    return null;
  }
  const registry = sanitizeRegistryUrl(context.registryUrl) ??
    `${hostOf(context.repositoryUrl)}/${context.repositoryName}`;
  const isDigest = /^[a-z0-9]+:/.test(reference);
  return {
    registry,
    loginHost: registry.split('/')[0],
    image,
    reference,
    isDigest,
    ref: `${registry}/${image}${isDigest ? '@' : ':'}${reference}`,
    plainHttp: isPlainHttp(context.repositoryUrl) ? ' --plain-http' : '',
  };
}

/**
 * A container image: pull, sign in, Dockerfile, Compose, Kubernetes, the artifacts attached to it and,
 * for a hosted repository, push.
 */
function imageSnippets(context: UsageContext, target: Target): UsageSnippet[] {
  const { registry, image, reference, ref, isDigest } = target;
  const strings = usageStrings();
  const classic = dockerSnippets(
    { format: 'docker', group: '', name: image, version: reference },
    { registryUrl: registry },
  );
  const byName = (displayName: string) =>
    (classic.find((snippet) => snippet.displayName === displayName)?.snippetText ?? '')
      .split(`${registry}/${image}:${reference}`).join(ref);

  const snippets: UsageSnippet[] = [
    { id: 'pull', label: 'docker pull', snippetText: `docker pull ${ref}` },
    {
      id: 'login',
      label: 'docker login',
      snippetText: `docker login ${target.loginHost}`,
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
    {
      id: 'discover',
      label: 'oras discover',
      snippetText: `oras discover ${ref}${target.plainHttp}`,
      description: strings.HINTS.ORAS_DISCOVER,
    },
  ];
  if (isHosted(context.repositoryType) && !isDigest) {
    snippets.push({
      id: 'push',
      label: 'docker push',
      snippetText: `docker tag ${image}:${reference} ${ref}\ndocker push ${ref}`,
      description: strings.HINTS.DOCKER_PUSH,
    });
  }
  return snippets;
}

/**
 * A Helm chart: pull, install, sign in and, for a hosted repository, push. Helm addresses a chart by the
 * repository it lives in and takes the version apart.
 */
function helmSnippets(context: UsageContext, target: Target): UsageSnippet[] {
  const { registry, image, reference, isDigest, plainHttp } = target;
  const strings = usageStrings();
  const chart = image.substring(image.lastIndexOf('/') + 1);
  const parent = image.includes('/') ? `/${image.substring(0, image.lastIndexOf('/'))}` : '';
  const address = isDigest
    ? `oci://${registry}/${image}@${reference}`
    : `oci://${registry}/${image} --version ${reference}`;

  const snippets: UsageSnippet[] = [
    {
      id: 'helm-pull',
      label: 'helm pull',
      snippetText: `helm pull ${address}${plainHttp}`,
      description: strings.HINTS.HELM_OCI_PULL,
    },
    {
      id: 'helm-install',
      label: 'helm install',
      snippetText: `helm install ${chart} ${address}${plainHttp}`,
      description: strings.HINTS.HELM_OCI_INSTALL,
    },
    {
      id: 'helm-login',
      label: 'helm registry login',
      snippetText: `helm registry login ${target.loginHost}${plainHttp}`,
      description: strings.HINTS.HELM_LOGIN,
    },
  ];
  if (isHosted(context.repositoryType) && !isDigest) {
    snippets.push({
      id: 'helm-push',
      label: 'helm push',
      snippetText: `helm package ./${chart}\nhelm push ${chart}-${reference}.tgz oci://${registry}${parent}${plainHttp}`,
      description: strings.HINTS.HELM_OCI_PUSH,
    });
  }
  return snippets;
}

/**
 * Any other OCI artifact (signature, SBOM, attestation, Wasm module, AI model...): oras pulls its files and
 * shows its manifest; an artifact attached to a subject leads to the other artifacts of that subject; a
 * hosted repository takes new artifacts with oras push or, for a subject, oras attach.
 */
function artifactSnippets(context: UsageContext, target: Target): UsageSnippet[] {
  const { registry, image, ref, isDigest, plainHttp } = target;
  const strings = usageStrings();
  const docker = dockerAttributesOf(context.attributes) ?? {};
  const subject = typeof docker.subject === 'string' && docker.subject ? docker.subject : null;
  const artifactType = typeof docker.artifactType === 'string' && docker.artifactType
    ? docker.artifactType
    : 'application/vnd.example.artifact';

  const snippets: UsageSnippet[] = [
    {
      id: 'oras-pull',
      label: 'oras pull',
      snippetText: `oras pull ${ref}${plainHttp}`,
      description: strings.HINTS.ORAS_PULL,
    },
    {
      id: 'oras-manifest',
      label: 'oras manifest fetch',
      snippetText: `oras manifest fetch --pretty ${ref}${plainHttp}`,
      description: strings.HINTS.ORAS_MANIFEST,
    },
  ];
  if (subject) {
    snippets.push({
      id: 'oras-discover',
      label: 'oras discover',
      snippetText: `oras discover ${registry}/${image}@${subject}${plainHttp}`,
      description: strings.HINTS.ORAS_DISCOVER_SUBJECT,
    });
  }
  snippets.push({
    id: 'oras-login',
    label: 'oras login',
    snippetText: `oras login ${target.loginHost}${plainHttp}`,
    description: strings.HINTS.ORAS_LOGIN,
  });
  if (isHosted(context.repositoryType)) {
    if (subject) {
      snippets.push({
        id: 'oras-attach',
        label: 'oras attach',
        snippetText: `oras attach --artifact-type ${artifactType} ${registry}/${image}@${subject} ./file${plainHttp}`,
        description: strings.HINTS.ORAS_ATTACH,
      });
    } else if (!isDigest) {
      snippets.push({
        id: 'oras-push',
        label: 'oras push',
        snippetText: `oras push --artifact-type ${artifactType} ${ref} ./file:application/octet-stream${plainHttp}`,
        description: strings.HINTS.ORAS_PUSH,
      });
    }
  }
  return snippets;
}

/**
 * Docker usage, chosen by what the manifest carries: the Docker clients for a container image, helm for a
 * Helm chart, oras for any other OCI artifact.
 *
 * The registry is the one the server computed for the repository; without it the repository is
 * addressed by path on the server's own host, which is how it is reached when no connector is set.
 */
export const generate: UsageGenerator = (context) => {
  const target = targetOf(context);
  if (!target) {
    return [];
  }
  const kind = dockerArtifactKind(context.attributes) ?? 'image';
  if (kind === 'image') {
    return imageSnippets(context, target);
  }
  if (kind === 'helm') {
    return helmSnippets(context, target);
  }
  return artifactSnippets(context, target);
};
