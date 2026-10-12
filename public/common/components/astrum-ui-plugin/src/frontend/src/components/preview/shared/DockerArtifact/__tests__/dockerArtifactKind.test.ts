import { dockerArtifactKind, dockerArtifactKindLabel, dockerAttributesOf } from '../dockerArtifactKind';

describe('dockerArtifactKind', () => {
  it('reads the kind the server recorded', () => {
    expect(dockerArtifactKind({ docker: { artifactKind: 'sbom' } })).toBe('sbom');
    expect(dockerArtifactKind({ docker: { artifactKind: 'HELM' } })).toBe('helm');
  });

  it('tells the kind of a manifest stored before kinds were recorded from its artifact type', () => {
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.cncf.helm.config.v1+json' } })).toBe('helm');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.dev.cosign.artifact.sig.v1+json' } }))
      .toBe('signature');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.cncf.notary.signature' } })).toBe('signature');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/spdx+json' } })).toBe('sbom');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.cyclonedx+json' } })).toBe('sbom');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.in-toto+json' } })).toBe('attestation');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.wasm.config.v0+json' } })).toBe('wasm');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.cncf.model.manifest.v1+json' } })).toBe('model');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.oci.image.config.v1+json' } })).toBe('image');
    expect(dockerArtifactKind({ docker: { artifactType: 'application/vnd.example.thing' } })).toBe('other');
  });

  it('treats an index or a manifest without an artifact type as an image', () => {
    expect(dockerArtifactKind({ docker: { mediaType: 'application/vnd.oci.image.index.v1+json' } })).toBe('image');
    expect(dockerArtifactKind({ docker: { mediaType: 'application/vnd.docker.distribution.manifest.v2+json' } }))
      .toBe('image');
  });

  it('knows nothing about assets without Docker attributes', () => {
    expect(dockerArtifactKind(undefined)).toBeNull();
    expect(dockerArtifactKind({ checksum: { sha1: 'x' } })).toBeNull();
    expect(dockerArtifactKind({ docker: {} })).toBeNull();
  });

  it('accepts the Docker attributes themselves', () => {
    expect(dockerAttributesOf({ digest: 'sha256:1' })).toEqual({ digest: 'sha256:1' });
    expect(dockerArtifactKind({ artifactKind: 'model' })).toBe('model');
  });

  it('labels the kinds', () => {
    expect(dockerArtifactKindLabel('helm')).toBe('Helm chart');
    expect(dockerArtifactKindLabel('other')).toBe('OCI artifact');
  });
});
