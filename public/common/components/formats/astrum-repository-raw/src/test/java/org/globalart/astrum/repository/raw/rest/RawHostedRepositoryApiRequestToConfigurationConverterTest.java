
package org.globalart.astrum.repository.raw.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.raw.ContentDisposition;
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
 * Tests for {@link RawHostedRepositoryApiRequestToConfigurationConverter}.
 */
@ExtendWith(MockitoExtension.class)
class RawHostedRepositoryApiRequestToConfigurationConverterTest
{
  @Mock
  private ConfigurationStore configurationStore;

  @Mock
  private RepositoryManager repositoryManager;

  private RawHostedRepositoryApiRequestToConfigurationConverter underTest;

  @BeforeEach
  void setUp() {
    when(configurationStore.newConfiguration()).thenReturn(new ConfigurationData());
    underTest = new RawHostedRepositoryApiRequestToConfigurationConverter();
    underTest.setConfigurationStore(configurationStore);
    underTest.setRepositoryManager(repositoryManager);
  }

  @Test
  void testConvert_defaultsContentDispositionToAttachment() {
    RawHostedRepositoryApiRequest request = createRequest(null);
    when(repositoryManager.get("raw-hosted")).thenReturn(null);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_defaultsContentDispositionToAttachmentWhenRawIsNull() {
    RawHostedRepositoryApiRequest request = createRequestWithNullRaw();
    when(repositoryManager.get("raw-hosted")).thenReturn(null);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_preservesExplicitContentDisposition() {
    RawHostedRepositoryApiRequest request = createRequest(ContentDisposition.INLINE);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.INLINE.name()));
  }

  @Test
  void testConvert_preservesExistingAttachmentOnUpdate() {
    RawHostedRepositoryApiRequest request = createRequest(null);
    mockExistingRepository("raw-hosted", "raw", ContentDisposition.ATTACHMENT.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_preservesExistingInlineOnUpdate() {
    RawHostedRepositoryApiRequest request = createRequest(null);
    mockExistingRepository("raw-hosted", "raw", ContentDisposition.INLINE.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.INLINE.name()));
  }

  @Test
  void testConvert_preservesNullOnUpdateForLegacyRepo() {
    RawHostedRepositoryApiRequest request = createRequest(null);
    mockLegacyRepository("raw-hosted", "raw");

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
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

  private void mockLegacyRepository(String name, String formatKey) {
    Configuration existingConfig = new ConfigurationData();
    existingConfig.setRepositoryName(name);
    // No contentDisposition set (legacy repo)

    Repository repo = mock(Repository.class);
    when(repo.getConfiguration()).thenReturn(existingConfig);
    when(repositoryManager.get(name)).thenReturn(repo);
  }

  private RawHostedRepositoryApiRequest createRequest(ContentDisposition contentDisposition) {
    RawAttributes raw = new RawAttributes(contentDisposition);
    HostedStorageAttributes storage = new HostedStorageAttributes("default", true, "ALLOW_ONCE");

    return new RawHostedRepositoryApiRequest(
        "raw-hosted",
        true,
        storage,
        null,
        raw,
        null);
  }

  private RawHostedRepositoryApiRequest createRequestWithNullRaw() {
    HostedStorageAttributes storage = new HostedStorageAttributes("default", true, "ALLOW_ONCE");

    return new RawHostedRepositoryApiRequest(
        "raw-hosted",
        true,
        storage,
        null,
        null,
        null);
  }
}
