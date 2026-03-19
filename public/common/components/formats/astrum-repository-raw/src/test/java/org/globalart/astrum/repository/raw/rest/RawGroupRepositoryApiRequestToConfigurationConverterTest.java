
package org.globalart.astrum.repository.raw.rest;

import java.util.Arrays;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.raw.ContentDisposition;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

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
 * Tests for {@link RawGroupRepositoryApiRequestToConfigurationConverter}.
 */
@ExtendWith(MockitoExtension.class)
class RawGroupRepositoryApiRequestToConfigurationConverterTest
{
  @Mock
  private ConfigurationStore configurationStore;

  @Mock
  private RepositoryManager repositoryManager;

  private RawGroupRepositoryApiRequestToConfigurationConverter underTest;

  @BeforeEach
  void setUp() {
    when(configurationStore.newConfiguration()).thenReturn(new ConfigurationData());
    underTest = new RawGroupRepositoryApiRequestToConfigurationConverter();
    underTest.setConfigurationStore(configurationStore);
    underTest.setRepositoryManager(repositoryManager);
  }

  @Test
  void testConvert_defaultsContentDispositionToAttachment() {
    RawGroupRepositoryApiRequest request = createRequest(null);
    when(repositoryManager.get("raw-group")).thenReturn(null);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_defaultsContentDispositionToAttachmentWhenRawIsNull() {
    RawGroupRepositoryApiRequest request = createRequestWithNullRaw();
    when(repositoryManager.get("raw-group")).thenReturn(null);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_preservesExplicitContentDisposition() {
    RawGroupRepositoryApiRequest request = createRequest(ContentDisposition.INLINE);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.INLINE.name()));
  }

  @Test
  void testConvert_preservesExistingAttachmentOnUpdate() {
    RawGroupRepositoryApiRequest request = createRequest(null);
    mockExistingRepository("raw-group", "raw", ContentDisposition.ATTACHMENT.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_preservesExistingInlineOnUpdate() {
    RawGroupRepositoryApiRequest request = createRequest(null);
    mockExistingRepository("raw-group", "raw", ContentDisposition.INLINE.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.INLINE.name()));
  }

  @Test
  void testConvert_preservesNullOnUpdateForLegacyRepo() {
    RawGroupRepositoryApiRequest request = createRequest(null);
    mockLegacyRepository("raw-group");

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

  private void mockLegacyRepository(String name) {
    Configuration existingConfig = new ConfigurationData();
    existingConfig.setRepositoryName(name);
    // No contentDisposition set (legacy repo)

    Repository repo = mock(Repository.class);
    when(repo.getConfiguration()).thenReturn(existingConfig);
    when(repositoryManager.get(name)).thenReturn(repo);
  }

  private RawGroupRepositoryApiRequest createRequest(ContentDisposition contentDisposition) {
    RawAttributes raw = new RawAttributes(contentDisposition);
    StorageAttributes storage = new StorageAttributes("default", true);
    GroupAttributes group = new GroupAttributes(Arrays.asList("raw-hosted", "raw-proxy"));

    return new RawGroupRepositoryApiRequest(
        "raw-group",
        true,
        storage,
        group,
        raw);
  }

  private RawGroupRepositoryApiRequest createRequestWithNullRaw() {
    StorageAttributes storage = new StorageAttributes("default", true);
    GroupAttributes group = new GroupAttributes(Arrays.asList("raw-hosted", "raw-proxy"));

    return new RawGroupRepositoryApiRequest(
        "raw-group",
        true,
        storage,
        group,
        null);
  }
}
