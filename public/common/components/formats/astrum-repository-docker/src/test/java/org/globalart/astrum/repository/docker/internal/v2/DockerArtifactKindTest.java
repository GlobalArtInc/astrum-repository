package org.globalart.astrum.repository.docker.internal.v2;

import java.nio.charset.StandardCharsets;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

public class DockerArtifactKindTest
{
  private static final String DIGEST = "sha256:" + "a".repeat(64);

  private static final String SUBJECT = "sha256:" + "b".repeat(64);

  private static DockerManifest manifest(
      final String artifactType,
      final String configType,
      final String... layerTypes) throws Exception
  {
    StringBuilder json = new StringBuilder("{\"schemaVersion\":2,\"mediaType\":\"")
        .append(DockerManifest.OCI_MANIFEST).append('"');
    if (artifactType != null) {
      json.append(",\"artifactType\":\"").append(artifactType).append('"');
    }
    json.append(",\"config\":{\"mediaType\":\"").append(configType).append("\",\"digest\":\"").append(DIGEST)
        .append("\",\"size\":2},\"layers\":[");
    for (int i = 0; i < layerTypes.length; i++) {
      json.append(i == 0 ? "" : ",").append("{\"mediaType\":\"").append(layerTypes[i]).append("\",\"digest\":\"")
          .append(DIGEST).append("\",\"size\":1}");
    }
    json.append("]}");
    return DockerManifest.parse(json.toString().getBytes(StandardCharsets.UTF_8), null);
  }

  @Test
  public void containerImages() throws Exception {
    assertThat(DockerArtifactKind.of(manifest(null, DockerArtifactKind.OCI_IMAGE_CONFIG,
        "application/vnd.oci.image.layer.v1.tar+gzip")), is(DockerArtifactKind.IMAGE));
    assertThat(DockerArtifactKind.of(manifest(null, DockerArtifactKind.DOCKER_IMAGE_CONFIG,
        "application/vnd.docker.image.rootfs.diff.tar.gzip")), is(DockerArtifactKind.IMAGE));
    DockerManifest index = DockerManifest.parse(("{\"schemaVersion\":2,\"mediaType\":\"" + DockerManifest.OCI_INDEX
        + "\",\"manifests\":[{\"mediaType\":\"" + DockerManifest.OCI_MANIFEST + "\",\"digest\":\"" + DIGEST
        + "\",\"size\":1}]}").getBytes(StandardCharsets.UTF_8), null);
    assertThat(DockerArtifactKind.of(index), is(DockerArtifactKind.IMAGE));
  }

  @Test
  public void helmChart() throws Exception {
    assertThat(DockerArtifactKind.of(manifest(null, DockerArtifactKind.HELM_CONFIG,
        "application/vnd.cncf.helm.chart.content.v1.tar+gzip")), is(DockerArtifactKind.HELM));
  }

  @Test
  public void signatures() throws Exception {
    assertThat(DockerArtifactKind.of(manifest("application/vnd.dev.cosign.artifact.sig.v1+json",
        DockerManifest.OCI_EMPTY, "application/vnd.dev.cosign.simplesigning.v1+json")),
        is(DockerArtifactKind.SIGNATURE));
    assertThat(DockerArtifactKind.of(manifest("application/vnd.cncf.notary.signature", DockerManifest.OCI_EMPTY,
        "application/jose+json")), is(DockerArtifactKind.SIGNATURE));
    assertThat(DockerArtifactKind.of(manifest(null, DockerArtifactKind.OCI_IMAGE_CONFIG,
        "application/vnd.dev.cosign.simplesigning.v1+json")), is(DockerArtifactKind.SIGNATURE));
    assertThat(DockerArtifactKind.of(manifest("application/vnd.dev.sigstore.bundle.v0.3+json",
        DockerManifest.OCI_EMPTY, "application/vnd.dev.sigstore.bundle.v0.3+json")), is(DockerArtifactKind.SIGNATURE));
  }

  @Test
  public void sbomsAndAttestations() throws Exception {
    assertThat(DockerArtifactKind.of(manifest("application/spdx+json", DockerManifest.OCI_EMPTY,
        "application/spdx+json")), is(DockerArtifactKind.SBOM));
    assertThat(DockerArtifactKind.of(manifest("application/vnd.cyclonedx+json", DockerManifest.OCI_EMPTY,
        "application/vnd.cyclonedx+json")), is(DockerArtifactKind.SBOM));
    assertThat(DockerArtifactKind.of(manifest(null, DockerArtifactKind.OCI_IMAGE_CONFIG,
        "application/vnd.in-toto+json")), is(DockerArtifactKind.ATTESTATION));
    assertThat(DockerArtifactKind.of(manifest("application/vnd.dsse.envelope.v1+json", DockerManifest.OCI_EMPTY,
        "application/vnd.dsse.envelope.v1+json")), is(DockerArtifactKind.ATTESTATION));
  }

  @Test
  public void wasmAndModels() throws Exception {
    assertThat(DockerArtifactKind.of(manifest(null, "application/vnd.wasm.config.v0+json",
        "application/wasm")), is(DockerArtifactKind.WASM));
    assertThat(DockerArtifactKind.of(manifest("application/vnd.cncf.model.manifest.v1+json",
        "application/vnd.cncf.model.config.v1+json", "application/vnd.cncf.model.weight.v1.raw")),
        is(DockerArtifactKind.MODEL));
    assertThat(DockerArtifactKind.of(manifest(null, "application/vnd.docker.ai.model.config.v0.1+json",
        "application/vnd.docker.ai.gguf.v3")), is(DockerArtifactKind.MODEL));
  }

  @Test
  public void artifactTypeComesFirst() throws Exception {
    assertThat(DockerArtifactKind.of(manifest("application/spdx+json", DockerManifest.OCI_EMPTY,
        "application/vnd.dev.cosign.simplesigning.v1+json")), is(DockerArtifactKind.SBOM));
  }

  @Test
  public void anyOtherArtifact() throws Exception {
    assertThat(DockerArtifactKind.of(manifest("application/vnd.example.thing", DockerManifest.OCI_EMPTY,
        "text/plain")), is(DockerArtifactKind.OTHER));
    assertThat(DockerArtifactKind.of(manifest(null, "application/vnd.example.config+json", "text/plain")),
        is(DockerArtifactKind.OTHER));
  }

  @Test
  public void tagSchema() throws Exception {
    DockerManifest image = manifest(null, DockerArtifactKind.OCI_IMAGE_CONFIG, "application/octet-stream");
    String tag = "sha256-" + "b".repeat(64);
    assertThat(image.kind(tag + ".sig"), is(DockerArtifactKind.SIGNATURE));
    assertThat(image.kind(tag + ".att"), is(DockerArtifactKind.ATTESTATION));
    assertThat(image.kind(tag + ".sbom"), is(DockerArtifactKind.SBOM));
    assertThat(image.kind("v1.0"), is(DockerArtifactKind.IMAGE));
    assertThat(DockerArtifactKind.tagSchemaSubject(tag + ".sig"), is(SUBJECT));
    assertThat(DockerArtifactKind.tagSchemaSubject("latest"), nullValue());
    assertThat(DockerArtifactKind.fallbackTag(SUBJECT), is(tag));
    assertThat(DockerArtifactKind.fallbackTag("sha512:" + "c".repeat(128)).length(), is(128));
    assertThat(DockerArtifactKind.fallbackTag("nope"), nullValue());
  }

  @Test
  public void ids() {
    assertThat(DockerArtifactKind.fromId("helm"), is(DockerArtifactKind.HELM));
    assertThat(DockerArtifactKind.fromId("unknown"), nullValue());
    assertThat(DockerArtifactKind.SBOM.id(), is("sbom"));
  }
}
