
package org.globalart.astrum.repository.maven.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.maven.ContentDisposition;
import org.globalart.astrum.repository.maven.api.MavenAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import org.globalart.astrum.repository.Repository;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link MavenHostedRepositoryApiRequestToConfigurationConverter}.
 */
@ExtendWith(MockitoExtension.class)
class MavenHostedRepositoryApiRequestToConfigurationConverterTest
{
  @Mock
  private ConfigurationStore configurationStore;

  @Mock
  private RepositoryManager repositoryManager;

  private MavenHostedRepositoryApiRequestToConfigurationConverter underTest;

  @BeforeEach
  void setUp() {
    when(configurationStore.newConfiguration()).thenReturn(new ConfigurationData());
    underTest = new MavenHostedRepositoryApiRequestToConfigurationConverter();
    underTest.setConfigurationStore(configurationStore);
    underTest.setRepositoryManager(repositoryManager);
  }

  @Test
  void testConvert_defaultsContentDispositionToAttachment() {
    MavenHostedRepositoryApiRequest request = createRequest(null);
    when(repositoryManager.get("maven-hosted")).thenReturn(null);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("maven").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_preservesExplicitContentDisposition() {
    MavenHostedRepositoryApiRequest request = createRequest(ContentDisposition.INLINE.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("maven").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.INLINE.name()));
  }

  @Test
  void testConvert_preservesExistingAttachmentOnUpdate() {
    MavenHostedRepositoryApiRequest request = createRequest(null);
    mockExistingRepository("maven-hosted", "maven", ContentDisposition.ATTACHMENT.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("maven").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_preservesExistingInlineOnUpdate() {
    MavenHostedRepositoryApiRequest request = createRequest(null);
    mockExistingRepository("maven-hosted", "maven", ContentDisposition.INLINE.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("maven").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.INLINE.name()));
  }

  @Test
  void testConvert_preservesNullOnUpdateForLegacyRepo() {
    MavenHostedRepositoryApiRequest request = createRequest(null);
    mockLegacyRepository("maven-hosted");

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("maven").get("contentDisposition", String.class);
    // Legacy repo with null should preserve null for backward compatibility
    assertThat(contentDisposition, nullValue());
  }

  private void mockExistingRepository(String name, String formatKey, String contentDisposition) {
    Configuration existingConfig = new ConfigurationData();
    existingConfig.setRepositoryName(name);
    existingConfig.attributes(formatKey).set("contentDisposition", contentDisposition);

    Repository repo = mock(Repository.class);
    when(repo.getConfiguration()).thenReturn(existingConfig);
    when(repositoryManager.get(name)).thenReturn(repo);
  }

  private void mockLegacyRepository(String name) {
    Configuration existingConfig = new ConfigurationData();
    existingConfig.setRepositoryName(name);

    Repository repo = mock(Repository.class);
    when(repo.getConfiguration()).thenReturn(existingConfig);
    when(repositoryManager.get(name)).thenReturn(repo);
  }

  private MavenHostedRepositoryApiRequest createRequest(String contentDisposition) {
    MavenAttributes maven = new MavenAttributes("RELEASE", "STRICT", contentDisposition);
    HostedStorageAttributes storage = new HostedStorageAttributes("default", true, "ALLOW_ONCE");

    return new MavenHostedRepositoryApiRequest(
        "maven-hosted",
        true,
        storage,
        null,
        maven,
        null);
  }
}
