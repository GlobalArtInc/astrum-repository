package org.globalart.astrum.repository.docker.internal;

import java.net.URI;
import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.rest.api.AssetXODescriptor;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.docker.DockerContentFacet.ATTR_ARTIFACT_TYPE;
import static org.globalart.astrum.repository.docker.DockerContentFacet.ATTR_DIGEST;
import static org.globalart.astrum.repository.docker.DockerContentFacet.ATTR_MEDIA_TYPE;
import static org.globalart.astrum.repository.docker.DockerContentFacet.ATTR_NAME;
import static org.globalart.astrum.repository.docker.DockerContentFacet.ATTR_SUBJECT;

/**
 * Exposes the Docker attributes of an asset and the registry a Docker client pulls it from.
 *
 * The registry is the server's own host followed by the repository name when the repository is reachable by path,
 * which works wherever the user interface does; otherwise the host with the repository's connector port.
 */
@Component
@Qualifier(DockerFormat.NAME)
public class DockerAssetXODescriptor
    implements AssetXODescriptor
{
  private static final Set<String> ATTRIBUTE_KEYS =
      Set.of(ATTR_NAME, ATTR_DIGEST, ATTR_MEDIA_TYPE, ATTR_SUBJECT, ATTR_ARTIFACT_TYPE);

  @Override
  public Set<String> listExposedAttributeKeys() {
    return ATTRIBUTE_KEYS;
  }

  @Nullable
  @Override
  public String computeRegistryUrl(@Nullable final Repository repository) {
    if (repository == null || !BaseUrlHolder.isSet()) {
      return null;
    }
    return registryUrl(BaseUrlHolder.get(), repository);
  }

  @Nullable
  static String registryUrl(final String baseUrl, final Repository repository) {
    URI uri;
    try {
      uri = URI.create(baseUrl);
    }
    catch (IllegalArgumentException e) {
      return null;
    }
    if (uri.getHost() == null) {
      return null;
    }
    String path = uri.getRawPath() == null ? "" : uri.getRawPath().replaceAll("/+$", "");
    if (path.isEmpty() && DockerConfig.pathEnabled(repository)) {
      return uri.getRawAuthority() + "/" + repository.getName();
    }
    Integer port = DockerConfig.httpsPort(repository.getConfiguration());
    if (port == null) {
      port = DockerConfig.httpPort(repository.getConfiguration());
    }
    return port == null ? null : uri.getHost() + ":" + port;
  }
}
