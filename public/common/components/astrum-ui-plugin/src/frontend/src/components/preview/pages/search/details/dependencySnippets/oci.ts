

import type { DependencySnippet, SnippetGenerator } from './types';

/**
 * Split the fully-qualified component name into {authority, repo, imageName}.
 *
 * Handles three shapes:
 *   host:port/repo/name
 *   host/repo/name
 *   host:port/name
 *
 * host may itself contain dots (e.g. astrum.example.com).
 */
function parseComponent(name: string): { host: string; authority: string; repo: string; imageName: string } {
  const firstSlash = name.indexOf('/');
  const authority = firstSlash === -1 ? name : name.substring(0, firstSlash);
  const remainder = firstSlash === -1 ? '' : name.substring(firstSlash + 1);

  let host = authority;
  const colonIdx = authority.lastIndexOf(':');
  if (colonIdx > -1) {
    host = authority.substring(0, colonIdx);
  }

  let repo = '';
  let imageName = remainder;
  const remSlash = remainder.indexOf('/');
  if (remSlash > -1) {
    repo = remainder.substring(0, remSlash);
    imageName = remainder.substring(remSlash + 1);
  }

  return { host, authority, repo, imageName: imageName || remainder };
}

/**
 * OCI dependency snippet generator.
 *
 * Ported from Classic AS.oci.controller.OciDependencySnippetController. Emits per-tool snippets
 * parameterized by host, port and repository path for ORAS, Docker, Podman, buildx, Helm (OCI),
 * Skopeo, Cosign, OpenTofu and Kubernetes. Returns no snippets for a component missing name or
 * version.
 */
export const generate: SnippetGenerator = (component, asset): DependencySnippet[] => {
  const yamlIndent = '  ';
  const { name, version } = component;

  if (!name || !version) {
    return [];
  }

  const parts = parseComponent(name);
  const shortName = name.substring(name.lastIndexOf('/') + 1);

  const isLocalhost = parts.host === 'localhost' || parts.host === '127.0.0.1' || parts.host === '0.0.0.0';
  const plainHttpFlag = isLocalhost ? ' --plain-http' : '';

  const hostPort = parts.authority;
  const repoPath = parts.repo ? `/${parts.repo}` : '';
  const imageRef = `${name}:${version}`;
  const digest = asset?.digest;

  return [
    {
      displayName: 'OCI Pull (ORAS)',
      description: 'Download artifact using ORAS CLI',
      snippetText: `oras pull ${imageRef}${plainHttpFlag}`,
    },
    {
      displayName: 'ORAS Push',
      description: 'Push a local file as an OCI artifact',
      snippetText: `oras push ${hostPort}${repoPath}/${parts.imageName}:${version} ./file.tar${plainHttpFlag}`,
    },
    {
      displayName: 'Docker Pull',
      description: 'Pull OCI image using Docker CLI',
      snippetText: `docker pull ${imageRef}`,
    },
    {
      displayName: 'Podman Pull',
      description: 'Pull OCI image using Podman',
      snippetText: `podman pull ${hostPort}${repoPath}/${parts.imageName}:${version}`,
    },
    {
      // Deliberate divergence from Classic (which omits repoPath here): pushing without repoPath
      // would target a different path than every other tool pulls from (Podman Pull, Cosign Sign).
      // Classic's form is a bug — do not "correct" this back to match it. TODO: file a Classic ticket.
      displayName: 'Docker Buildx Push',
      description: 'Build a multi-platform image and push to this registry',
      snippetText: `docker buildx build --push --tag ${hostPort}${repoPath}/${parts.imageName}:${version} .`,
    },
    {
      displayName: 'Dockerfile',
      description: 'Use as base image in Dockerfile',
      snippetText: `FROM ${imageRef}`,
    },
    {
      displayName: 'Helm Push (OCI)',
      description: 'Push a packaged Helm chart to this OCI registry',
      snippetText: `helm push mychart-1.0.0.tgz oci://${hostPort}${repoPath}`,
    },
    {
      displayName: 'Helm Install (OCI)',
      description: 'Install a Helm chart served from this OCI registry',
      snippetText: `helm install myrel oci://${hostPort}${repoPath}/${parts.imageName} --version ${version}`,
    },
    {
      displayName: 'Helm Install (Bitnami charts)',
      description: 'Install a Bitnami chart proxied through this OCI repository',
      snippetText: `helm install myrel oci://${hostPort}/bitnamicharts/postgresql`,
    },
    {
      displayName: 'Skopeo Copy',
      description: 'Mirror an image from this registry to a local OCI layout',
      snippetText: `skopeo copy docker://${imageRef} oci:./${parts.imageName}:${version}`,
    },
    {
      displayName: 'Cosign Sign',
      description: 'Sign the image by digest with cosign',
      snippetText:
        'cosign sign ' +
        (digest
          ? `${hostPort}${repoPath}/${parts.imageName}@${digest}`
          : `${hostPort}${repoPath}/${parts.imageName}@<digest>`),
    },
    {
      displayName: 'Cosign Verify (keyless)',
      description: 'Verify a keyless cosign signature by OIDC identity',
      snippetText:
        `cosign verify ${imageRef} \\\n` +
        `${yamlIndent}--certificate-identity-regexp '.*' \\\n` +
        `${yamlIndent}--certificate-oidc-issuer-regexp '.*'`,
    },
    {
      displayName: 'OpenTofu (tofu init)',
      description: 'Point OpenTofu at this OCI registry for provider/module mirroring',
      snippetText:
        'tofu init\n' +
        '# configure in your .tofurc / CLI config:\n' +
        `oci_credentials "${hostPort}" {\n` +
        `${yamlIndent}username = "<astrum-user>"\n` +
        `${yamlIndent}password = "<astrum-token>"\n` +
        '}',
    },
    {
      displayName: 'Kubernetes',
      description: 'Use image in Kubernetes pod specification',
      snippetText:
        'spec:\n' +
        `${yamlIndent}containers:\n` +
        `${yamlIndent}- name: ${shortName}\n` +
        `${yamlIndent.repeat(2)}image: ${imageRef}`,
    },
  ];
};
