package org.globalart.astrum.repository.docker.internal;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.docker.DockerConfig;

import org.junit.After;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class DockerAssetXODescriptorTest
{
  private final DockerAssetXODescriptor descriptor = new DockerAssetXODescriptor();

  @After
  public void unsetBaseUrl() {
    BaseUrlHolder.unset();
  }

  private static Repository repository(final Map<String, Object> docker) {
    Configuration configuration = new ConfigurationData();
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    attributes.put(DockerConfig.DOCKER, new HashMap<>(docker));
    configuration.setAttributes(attributes);
    Repository repository = mock(Repository.class);
    when(repository.getName()).thenReturn("docker-onprem");
    when(repository.getConfiguration()).thenReturn(configuration);
    return repository;
  }

  @Test
  public void exposesTheDockerAttributes() {
    assertThat(descriptor.listExposedAttributeKeys(),
        containsInAnyOrder("name", "digest", "mediaType", "subject", "artifactType"));
  }

  @Test
  public void pullsByPathFromTheServerHost() {
    BaseUrlHolder.set("https://astrum.example.com/", "");

    assertThat(descriptor.computeRegistryUrl(repository(Map.of())), is("astrum.example.com/docker-onprem"));
  }

  @Test
  public void keepsANonDefaultPortOfTheServer() {
    assertThat(DockerAssetXODescriptor.registryUrl("http://localhost:8081", repository(Map.of())),
        is("localhost:8081/docker-onprem"));
  }

  @Test
  public void usesTheConnectorPortWhenPathRoutingIsOff() {
    Repository repository = repository(Map.of(DockerConfig.PATH_ENABLED, false, DockerConfig.HTTPS_PORT, 8443,
        DockerConfig.HTTP_PORT, 8082));

    assertThat(DockerAssetXODescriptor.registryUrl("https://astrum.example.com", repository),
        is("astrum.example.com:8443"));
  }

  @Test
  public void usesTheConnectorPortWhenTheServerIsNotAtTheRootOfItsHost() {
    Repository repository = repository(Map.of(DockerConfig.HTTP_PORT, "8082"));

    assertThat(DockerAssetXODescriptor.registryUrl("https://example.com/astrum", repository),
        is("example.com:8082"));
  }

  @Test
  public void hasNoRegistryWithoutPathRoutingOrConnector() {
    Repository repository = repository(Map.of(DockerConfig.PATH_ENABLED, false));

    assertThat(DockerAssetXODescriptor.registryUrl("https://astrum.example.com", repository), nullValue());
  }

  @Test
  public void hasNoRegistryOutsideARequest() {
    assertThat(descriptor.computeRegistryUrl(repository(Map.of())), nullValue());
  }
}
