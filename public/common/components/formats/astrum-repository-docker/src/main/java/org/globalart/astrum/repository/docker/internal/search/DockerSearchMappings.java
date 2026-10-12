package org.globalart.astrum.repository.docker.internal.search;

import java.util.List;

import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_2;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_3;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_4;
import static org.globalart.astrum.repository.rest.sql.SearchField.NAME;
import static org.globalart.astrum.repository.rest.sql.SearchField.VERSION;

/**
 * Search Docker tags by image name and tag, by the digest and media type of the manifest they point at, and by what
 * the manifest carries: {@code docker.artifactKind} is one of image, helm, signature, sbom, attestation, wasm, model
 * and other, {@code docker.artifactType} the artifact type or config media type of the manifest.
 */
@Component
@Qualifier(DockerFormat.NAME)
public class DockerSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = List.of(
      new SearchMapping("docker.imageName", "name.raw", "Docker image name", NAME),
      new SearchMapping("docker.imageTag", "version", "Docker image tag", VERSION),
      new SearchMapping("docker.contentDigest", "assets.attributes.docker.digest", "Docker manifest digest",
          FORMAT_FIELD_1, false),
      new SearchMapping("docker.mediaType", "assets.attributes.docker.mediaType", "Docker manifest media type",
          FORMAT_FIELD_2, false),
      new SearchMapping("docker.artifactType", "assets.attributes.docker.artifactType",
          "OCI artifact type or config media type", FORMAT_FIELD_3, false),
      new SearchMapping("docker.artifactKind", "assets.attributes.docker.artifactKind",
          "What the manifest carries: image, helm, signature, sbom, attestation, wasm, model or other",
          FORMAT_FIELD_4, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
