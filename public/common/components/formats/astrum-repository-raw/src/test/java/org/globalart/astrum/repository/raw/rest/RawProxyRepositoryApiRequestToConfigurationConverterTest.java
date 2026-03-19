
package org.globalart.astrum.repository.raw.rest;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.raw.ContentDisposition;
import org.globalart.astrum.repository.rest.api.model.HttpClientAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link RawProxyRepositoryApiRequestToConfigurationConverter}.
 */
@ExtendWith(MockitoExtension.class)
class RawProxyRepositoryApiRequestToConfigurationConverterTest
{
  @Mock
  private ConfigurationStore configurationStore;

  @Mock
  private RoutingRuleStore routingRuleStore;

  @Mock
  private RepositoryManager repositoryManager;

  private RawProxyRepositoryApiRequestToConfigurationConverter underTest;

  @BeforeEach
  void setUp() {
    when(configurationStore.newConfiguration()).thenReturn(new ConfigurationData());
    underTest = new RawProxyRepositoryApiRequestToConfigurationConverter(routingRuleStore, repositoryManager, true);
    underTest.setConfigurationStore(configurationStore);
  }

  @Test
  void testConvert_defaultsContentDispositionToAttachment() {
    RawProxyRepositoryApiRequest request = createRequest(null, null, null);
    when(repositoryManager.get("raw-proxy")).thenReturn(null);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_defaultsContentDispositionToAttachmentWhenRawIsNull() {
    RawProxyRepositoryApiRequest request = createRequestWithNullRaw();
    when(repositoryManager.get("raw-proxy")).thenReturn(null);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_preservesExplicitContentDisposition() {
    RawProxyRepositoryApiRequest request = createRequest(ContentDisposition.INLINE, null, null);

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.INLINE.name()));
  }

  @Test
  void testConvert_preservesExistingAttachmentOnUpdate() {
    RawProxyRepositoryApiRequest request = createRequest(null, null, null);
    mockExistingRepository("raw-proxy", "raw", ContentDisposition.ATTACHMENT.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.ATTACHMENT.name()));
  }

  @Test
  void testConvert_preservesExistingInlineOnUpdate() {
    RawProxyRepositoryApiRequest request = createRequest(null, null, null);
    mockExistingRepository("raw-proxy", "raw", ContentDisposition.INLINE.name());

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    assertThat(contentDisposition, is(ContentDisposition.INLINE.name()));
  }

  @Test
  void testConvert_preservesNullOnUpdateForLegacyRepo() {
    RawProxyRepositoryApiRequest request = createRequest(null, null, null);
    mockLegacyRepository("raw-proxy");

    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    String contentDisposition = config.attributes("raw").get("contentDisposition", String.class);
    // Legacy repo with null should preserve null for backward compatibility
    assertThat(contentDisposition, nullValue());
  }

  @Test
  void testConvert_withQueryParamsEnabled_setsAllFields() {
    RawProxyRepositoryApiRequest request =
        createRequest(ContentDisposition.INLINE, true, Arrays.asList("api_key", "token"));

    Configuration config = underTest.convert(request);

    NestedAttributesMap rawAttrs = config.attributes("raw");
    assertThat(rawAttrs.get("contentDisposition", String.class), is("INLINE"));
    assertThat(rawAttrs.get("forwardQueryParameters", Boolean.class), is(true));
    assertThat(rawAttrs.get("excludedQueryParameters", List.class), is(Arrays.asList("api_key", "token")));
  }

  @Test
  void testConvert_withNullQueryParamsFields_skipsOptionalFields() {
    RawProxyRepositoryApiRequest request = createRequest(ContentDisposition.ATTACHMENT, null, null);

    Configuration config = underTest.convert(request);

    NestedAttributesMap rawAttrs = config.attributes("raw");
    assertThat(rawAttrs.get("contentDisposition", String.class), is("ATTACHMENT"));
    assertThat(rawAttrs.get("forwardQueryParameters", Boolean.class), is(nullValue()));
    assertThat(rawAttrs.get("excludedQueryParameters", List.class), is(nullValue()));
  }

  @Test
  void testConvert_withEmptyExclusionList_setsEmptyList() {
    RawProxyRepositoryApiRequest request =
        createRequest(ContentDisposition.ATTACHMENT, false, Collections.emptyList());

    Configuration config = underTest.convert(request);

    NestedAttributesMap rawAttrs = config.attributes("raw");
    assertThat(rawAttrs.get("forwardQueryParameters", Boolean.class), is(false));
    assertThat(rawAttrs.get("excludedQueryParameters", List.class), is(Collections.emptyList()));
  }

  private void mockExistingRepository(String name, String formatKey, String contentDisposition) {
    Configuration existingConfig = new ConfigurationData();
    existingConfig.setRepositoryName(name);
    existingConfig.attributes(formatKey).set("contentDisposition", contentDisposition);

    Repository repo = mock(Repository.class);
    when(repo.getConfiguration()).thenReturn(existingConfig);
    when(repositoryManager.get(name)).thenReturn(repo);
  }

  private void mockLegacyRepository(final String name) {
    Configuration existingConfig = new ConfigurationData();
    existingConfig.setRepositoryName(name);
    // No contentDisposition set (legacy repo)

    Repository repo = mock(Repository.class);
    when(repo.getConfiguration()).thenReturn(existingConfig);
    when(repositoryManager.get(name)).thenReturn(repo);
  }

  private RawProxyRepositoryApiRequest createRequest(
      ContentDisposition contentDisposition,
      Boolean forwardQueryParameters,
      List<String> excludedQueryParameters)
  {
    RawAttributes raw = new RawAttributes(contentDisposition, forwardQueryParameters, excludedQueryParameters);
    StorageAttributes storage = new StorageAttributes("default", true);
    ProxyAttributes proxy = new ProxyAttributes("https://example.org/raw", 1440, 1440, false);
    NegativeCacheAttributes negativeCache = new NegativeCacheAttributes(false, 1440);
    HttpClientAttributes httpClient = new HttpClientAttributes(false, true, null, null);

    return new RawProxyRepositoryApiRequest(
        "raw-proxy",
        true,
        storage,
        null,
        proxy,
        negativeCache,
        httpClient,
        null,
        raw,
        null);
  }

  private RawProxyRepositoryApiRequest createRequestWithNullRaw() {
    StorageAttributes storage = new StorageAttributes("default", true);
    ProxyAttributes proxy = new ProxyAttributes("https://example.org/raw", 1440, 1440, false);
    NegativeCacheAttributes negativeCache = new NegativeCacheAttributes(false, 1440);
    HttpClientAttributes httpClient = new HttpClientAttributes(false, true, null, null);

    return new RawProxyRepositoryApiRequest(
        "raw-proxy",
        true,
        storage,
        null,
        proxy,
        negativeCache,
        httpClient,
        null,
        null,
        null);
  }
}
