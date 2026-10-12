import UIStrings from '../../../../constants/UIStrings';

/**
 * What a Docker manifest carries. A Docker repository stores any OCI artifact, so a manifest is not
 * always a container image.
 */
export type DockerArtifactKind = 'image' | 'helm' | 'signature' | 'sbom' | 'attestation' | 'wasm' | 'model' | 'other';

export const DOCKER_ARTIFACT_KINDS: readonly DockerArtifactKind[] = [
  'image', 'helm', 'signature', 'sbom', 'attestation', 'wasm', 'model', 'other',
];

const IMAGE_CONFIGS = new Set([
  'application/vnd.oci.image.config.v1+json',
  'application/vnd.docker.container.image.v1+json',
]);

const INDEXES = new Set([
  'application/vnd.oci.image.index.v1+json',
  'application/vnd.docker.distribution.manifest.list.v2+json',
]);

/**
 * The kind of a type that tells it, read the way the server reads it: signatures, attestations and SBOMs
 * before Helm charts, Wasm modules and models.
 */
function kindOfType(type: string): DockerArtifactKind | null {
  if (type.startsWith('application/vnd.dev.cosign.artifact.sig') || type.includes('simplesigning')
    || type.startsWith('application/vnd.cncf.notary') || type.startsWith('application/vnd.dev.sigstore.bundle')) {
    return 'signature';
  }
  if (type.includes('in-toto') || type.startsWith('application/vnd.dsse.envelope')) {
    return 'attestation';
  }
  if (type.includes('spdx') || type.includes('cyclonedx') || type.includes('syft')
    || type.startsWith('application/vnd.dev.cosign.artifact.sbom')) {
    return 'sbom';
  }
  if (type.startsWith('application/vnd.cncf.helm.')) {
    return 'helm';
  }
  if (type.includes('wasm')) {
    return 'wasm';
  }
  if (type.startsWith('application/vnd.cncf.model.') || type.startsWith('application/vnd.docker.ai.')
    || type.startsWith('application/vnd.ollama.image.')) {
    return 'model';
  }
  return null;
}

function text(value: unknown): string | null {
  return typeof value === 'string' && value.trim() !== '' ? value.trim().toLowerCase() : null;
}

/**
 * The Docker attributes of an asset: either the attributes bag of the asset (`{ docker: {...} }`) or the
 * Docker attributes themselves.
 */
export function dockerAttributesOf(attributes: unknown): Record<string, unknown> | null {
  if (!attributes || typeof attributes !== 'object') {
    return null;
  }
  const bag = attributes as Record<string, unknown>;
  const docker = bag.docker;
  if (docker && typeof docker === 'object') {
    return docker as Record<string, unknown>;
  }
  return 'digest' in bag || 'mediaType' in bag || 'artifactKind' in bag ? bag : null;
}

/**
 * The kind of a Docker manifest from its attributes. The server records it since it tells artifacts apart;
 * for a manifest stored before, it is read from the artifact type and media type.
 */
export function dockerArtifactKind(attributes: unknown): DockerArtifactKind | null {
  const docker = dockerAttributesOf(attributes);
  if (!docker) {
    return null;
  }
  const recorded = text(docker.artifactKind);
  if (recorded && (DOCKER_ARTIFACT_KINDS as readonly string[]).includes(recorded)) {
    return recorded as DockerArtifactKind;
  }
  const artifactType = text(docker.artifactType);
  const mediaType = text(docker.mediaType);
  if (!artifactType && !mediaType) {
    return null;
  }
  if (artifactType) {
    const byType = kindOfType(artifactType);
    if (byType) {
      return byType;
    }
    return IMAGE_CONFIGS.has(artifactType) ? 'image' : 'other';
  }
  return mediaType && (INDEXES.has(mediaType) || mediaType.includes('manifest')) ? 'image' : 'other';
}

/**
 * The label of a kind, e.g. "Helm chart".
 */
export function dockerArtifactKindLabel(kind: DockerArtifactKind): string {
  return UIStrings.BROWSE.DOCKER_KINDS[kind];
}
