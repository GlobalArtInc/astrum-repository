package org.globalart.astrum.repository.swift.rest;

import java.util.List;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationConstants;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.rest.api.model.GroupDeployAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.HttpClientAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class SwiftRepositoryApiRequestToConfigurationConverterTest
{
  private final ConfigurationStore configurationStore = mock(ConfigurationStore.class);

  @BeforeEach
  void setUp() {
    when(configurationStore.newConfiguration()).thenAnswer(invocation -> new ConfigurationData());
  }

  @Test
  void convertsHostedRequests() {
    SwiftHostedRepositoryApiRequestToConfigurationConverter converter =
        new SwiftHostedRepositoryApiRequestToConfigurationConverter();
    converter.setConfigurationStore(configurationStore);
    Configuration configuration = converter.convert(new SwiftHostedRepositoryApiRequest("swift-hosted", true,
        new HostedStorageAttributes("default", true, "allow_once"), null, null));
    assertThat(configuration.getRecipeName(), is("swift-hosted"));
    assertThat(configuration.getRepositoryName(), is("swift-hosted"));
    assertThat(configuration.attributes("storage").get("writePolicy"), is("ALLOW_ONCE"));
  }

  @Test
  void convertsProxyRequests() {
    SwiftProxyRepositoryApiRequestToConfigurationConverter converter =
        new SwiftProxyRepositoryApiRequestToConfigurationConverter(mock(RoutingRuleStore.class));
    converter.setConfigurationStore(configurationStore);
    Configuration configuration = converter.convert(new SwiftProxyRepositoryApiRequest("swift-proxy", true,
        new StorageAttributes("default", true), null, new ProxyAttributes("https://registry.example.com/", 1440, 60, false),
        new NegativeCacheAttributes(true, 1440), new HttpClientAttributes(false, true, null, null), null, null));
    assertThat(configuration.getRecipeName(), is("swift-proxy"));
    assertThat(configuration.attributes("proxy").get("remoteUrl"), is("https://registry.example.com/"));
  }

  @Test
  void convertsGroupRequestsWithAWritableMember() {
    SwiftGroupRepositoryApiRequestToConfigurationConverter converter =
        new SwiftGroupRepositoryApiRequestToConfigurationConverter();
    converter.setConfigurationStore(configurationStore);
    Configuration configuration = converter.convert(new SwiftGroupRepositoryApiRequest("swift-group", true,
        new StorageAttributes("default", true), new GroupDeployAttributes(List.of("swift-hosted", "swift-proxy"),
        "swift-hosted")));
    assertThat(configuration.getRecipeName(), is("swift-group"));
    assertThat(configuration.attributes("group").get("memberNames"), is(List.of("swift-hosted", "swift-proxy")));
    assertThat(configuration.attributes("group").get(ConfigurationConstants.GROUP_WRITE_MEMBER), is("swift-hosted"));
  }
}
