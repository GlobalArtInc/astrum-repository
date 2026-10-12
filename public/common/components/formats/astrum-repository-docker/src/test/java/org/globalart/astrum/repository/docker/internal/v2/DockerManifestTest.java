package org.globalart.astrum.repository.docker.internal.v2;

import java.nio.charset.StandardCharsets;
import java.util.Map;

import org.globalart.astrum.repository.docker.internal.v2.DockerManifest.InvalidManifestException;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

public class DockerManifestTest
{
  private static final String CONFIG = "sha256:" + "c".repeat(64);

  private static final String LAYER = "sha256:" + "1".repeat(64);

  private static final String FOREIGN = "sha256:" + "f".repeat(64);

  private static final String CHILD = "sha256:" + "2".repeat(64);

  private static byte[] bytes(final String json) {
    return json.getBytes(StandardCharsets.UTF_8);
  }

  @Test
  public void dockerImageManifestReferencesConfigAndLayers() throws Exception {
    DockerManifest manifest = DockerManifest.parse(bytes("{\"schemaVersion\":2,\"mediaType\":\""
        + DockerManifest.DOCKER_V2 + "\",\"config\":{\"digest\":\"" + CONFIG + "\"},\"layers\":[{\"digest\":\""
        + LAYER + "\"},{\"mediaType\":\"application/vnd.docker.image.rootfs.foreign.diff.tar.gzip\",\"digest\":\""
        + FOREIGN + "\",\"urls\":[\"https://example.com/layer\"]}]}"), DockerManifest.DOCKER_V2);
    assertThat(manifest.getMediaType(), is(DockerManifest.DOCKER_V2));
    assertThat(manifest.isIndex(), is(false));
    assertThat(manifest.getBlobs(), contains(CONFIG, LAYER));
    assertThat(manifest.getManifests(), empty());
  }

  @Test
  public void ociIndexWithoutMediaTypeUsesHeader() throws Exception {
    DockerManifest manifest = DockerManifest.parse(
        bytes("{\"schemaVersion\":2,\"manifests\":[{\"digest\":\"" + CHILD + "\"}]}"),
        DockerManifest.OCI_INDEX + "; charset=utf-8");
    assertThat(manifest.getMediaType(), is(DockerManifest.OCI_INDEX));
    assertThat(manifest.isIndex(), is(true));
    assertThat(manifest.getManifests(), contains(CHILD));
    assertThat(manifest.getBlobs(), empty());
  }

  @Test
  public void genericContentTypeFallsBackToBody() throws Exception {
    DockerManifest manifest = DockerManifest.parse(bytes("{\"schemaVersion\":2,\"config\":{\"digest\":\"" + CONFIG
        + "\"},\"layers\":[]}"), "application/json");
    assertThat(manifest.getMediaType(), is(DockerManifest.OCI_MANIFEST));
  }

  @Test
  public void subjectAndArtifactType() throws Exception {
    DockerManifest manifest = DockerManifest.parse(bytes("{\"schemaVersion\":2,\"mediaType\":\""
        + DockerManifest.OCI_MANIFEST + "\",\"artifactType\":\"application/vnd.example\",\"config\":{\"mediaType\":"
        + "\"application/vnd.oci.empty.v1+json\",\"digest\":\"" + CONFIG + "\"},\"layers\":[],\"subject\":{"
        + "\"digest\":\"" + CHILD + "\"}}"), null);
    assertThat(manifest.getSubject(), is(CHILD));
    assertThat(manifest.getArtifactType(), is("application/vnd.example"));
  }

  @Test
  public void configLayersAndAnnotationsAreRead() throws Exception {
    DockerManifest manifest = DockerManifest.parse(bytes("{\"schemaVersion\":2,\"mediaType\":\""
        + DockerManifest.OCI_MANIFEST + "\",\"config\":{\"mediaType\":\"application/vnd.cncf.helm.config.v1+json\","
        + "\"digest\":\"" + CONFIG + "\"},\"layers\":[{\"mediaType\":"
        + "\"application/vnd.cncf.helm.chart.content.v1.tar+gzip\",\"digest\":\"" + LAYER + "\"}],"
        + "\"annotations\":{\"org.opencontainers.image.title\":\"chart\",\"ignored\":1}}"), null);
    assertThat(manifest.getConfigMediaType(), is("application/vnd.cncf.helm.config.v1+json"));
    assertThat(manifest.getLayerMediaTypes(), contains("application/vnd.cncf.helm.chart.content.v1.tar+gzip"));
    assertThat(manifest.getAnnotations(), is(Map.of("org.opencontainers.image.title", "chart")));
    assertThat(manifest.getExplicitArtifactType(), nullValue());
    assertThat(manifest.getArtifactType(), is("application/vnd.cncf.helm.config.v1+json"));
    assertThat(manifest.kind(null), is(DockerArtifactKind.HELM));
  }

  @Test
  public void indexHasNoImplicitArtifactType() throws Exception {
    DockerManifest manifest = DockerManifest.parse(bytes("{\"schemaVersion\":2,\"mediaType\":\""
        + DockerManifest.OCI_INDEX + "\",\"manifests\":[]}"), null);
    assertThat(manifest.getArtifactType(), nullValue());
  }

  @Test
  public void artifactManifestListsBlobs() throws Exception {
    DockerManifest manifest = DockerManifest.parse(bytes("{\"mediaType\":\"" + DockerManifest.OCI_ARTIFACT_MANIFEST
        + "\",\"schemaVersion\":2,\"artifactType\":\"application/vnd.example\",\"blobs\":[{\"digest\":\""
        + LAYER + "\"}]}"), null);
    assertThat(manifest.getBlobs(), contains(LAYER));
    assertThat(manifest.getArtifactType(), is("application/vnd.example"));
  }

  @Test(expected = InvalidManifestException.class)
  public void subjectMustBeADescriptor() throws Exception {
    DockerManifest.parse(bytes("{\"schemaVersion\":2,\"layers\":[],\"subject\":\"" + CHILD + "\"}"), null);
  }

  @Test(expected = InvalidManifestException.class)
  public void layersMustBeAnArray() throws Exception {
    DockerManifest.parse(bytes("{\"schemaVersion\":2,\"layers\":{}}"), null);
  }

  @Test
  public void schema1IsRecognised() throws Exception {
    DockerManifest manifest = DockerManifest.parse(bytes("{\"schemaVersion\":1,\"fsLayers\":[{\"blobSum\":\""
        + LAYER + "\"}],\"signatures\":[]}"), null);
    assertThat(manifest.isSchema1(), is(true));
    assertThat(manifest.getMediaType(), is(DockerManifest.DOCKER_V1_SIGNED));
    assertThat(manifest.getSubject(), nullValue());
  }

  @Test(expected = InvalidManifestException.class)
  public void mismatchedContentTypeIsRejected() throws Exception {
    DockerManifest.parse(bytes("{\"schemaVersion\":2,\"mediaType\":\"" + DockerManifest.OCI_MANIFEST
        + "\",\"layers\":[]}"), DockerManifest.DOCKER_V2);
  }

  @Test(expected = InvalidManifestException.class)
  public void invalidDigestIsRejected() throws Exception {
    DockerManifest.parse(bytes("{\"schemaVersion\":2,\"layers\":[{\"digest\":\"nope\"}]}"), null);
  }

  @Test(expected = InvalidManifestException.class)
  public void invalidJsonIsRejected() throws Exception {
    DockerManifest.parse(bytes("not json"), null);
  }
}
