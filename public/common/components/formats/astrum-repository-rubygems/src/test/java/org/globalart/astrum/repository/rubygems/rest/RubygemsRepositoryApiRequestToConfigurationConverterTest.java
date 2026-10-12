package org.globalart.astrum.repository.rubygems.rest;

import java.util.List;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
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
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.lenient;

@ExtendWith(MockitoExtension.class)
class RubygemsRepositoryApiRequestToConfigurationConverterTest
{
  @Mock
  private ConfigurationStore configurationStore;

  @Mock
  private RoutingRuleStore routingRuleStore;

  @BeforeEach
  void setUp() {
    lenient().when(configurationStore.newConfiguration()).thenAnswer(invocation -> new ConfigurationData());
  }

  @Test
  void convertsHostedRequests() {
    RubygemsHostedRepositoryApiRequestToConfigurationConverter underTest =
        new RubygemsHostedRepositoryApiRequestToConfigurationConverter();
    underTest.setConfigurationStore(configurationStore);

    Configuration configuration = underTest.convert(new RubygemsHostedRepositoryApiRequest("gems-hosted", true,
        new HostedStorageAttributes("default", true, "allow_once"), null, null));

    assertThat(configuration.getRepositoryName(), is("gems-hosted"));
    assertThat(configuration.getRecipeName(), is("rubygems-hosted"));
    assertThat(configuration.attributes("storage").get("writePolicy"), is("ALLOW_ONCE"));
  }

  @Test
  void convertsProxyRequests() {
    RubygemsProxyRepositoryApiRequestToConfigurationConverter underTest =
        new RubygemsProxyRepositoryApiRequestToConfigurationConverter(routingRuleStore);
    underTest.setConfigurationStore(configurationStore);

    Configuration configuration = underTest.convert(new RubygemsProxyRepositoryApiRequest("gems-proxy", true,
        new StorageAttributes("default", true), null, new ProxyAttributes("https://rubygems.org", 1440, 60, false),
        new NegativeCacheAttributes(true, 60), new HttpClientAttributes(false, true, null, null), null, null));

    assertThat(configuration.getRecipeName(), is("rubygems-proxy"));
    assertThat(configuration.attributes("proxy").get("remoteUrl"), is("https://rubygems.org"));
    assertThat(configuration.attributes("proxy").get("metadataMaxAge"), is(60));
  }

  @Test
  void convertsGroupRequests() {
    RubygemsGroupRepositoryApiRequestToConfigurationConverter underTest =
        new RubygemsGroupRepositoryApiRequestToConfigurationConverter();
    underTest.setConfigurationStore(configurationStore);

    Configuration configuration = underTest.convert(new RubygemsGroupRepositoryApiRequest("gems", true,
        new StorageAttributes("default", true), new GroupAttributes(List.of("gems-hosted", "gems-proxy"))));

    assertThat(configuration.getRecipeName(), is("rubygems-group"));
    @SuppressWarnings("unchecked")
    List<String> members = (List<String>) configuration.attributes("group").get("memberNames");
    assertThat(members, contains("gems-hosted", "gems-proxy"));
  }
}
