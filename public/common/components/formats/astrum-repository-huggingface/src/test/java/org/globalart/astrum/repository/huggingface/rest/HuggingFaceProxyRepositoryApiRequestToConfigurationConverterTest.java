package org.globalart.astrum.repository.huggingface.rest;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class HuggingFaceProxyRepositoryApiRequestToConfigurationConverterTest
{
  private static final String BODY = "{\"name\":\"hf-proxy\",\"online\":true,"
      + "\"storage\":{\"blobStoreName\":\"default\",\"strictContentTypeValidation\":true},"
      + "\"proxy\":{\"remoteUrl\":\"https://huggingface.co/\",\"contentMaxAge\":-1,\"metadataMaxAge\":60},"
      + "\"negativeCache\":{\"enabled\":true,\"timeToLive\":60},"
      + "\"httpClient\":{\"blocked\":false,\"autoBlock\":true,"
      + "\"authentication\":{\"type\":\"bearerToken\",\"bearerToken\":\"hf_example\"}}}";

  @Mock
  private ConfigurationStore configurationStore;

  @Mock
  private RoutingRuleStore routingRuleStore;

  private HuggingFaceProxyRepositoryApiRequestToConfigurationConverter underTest;

  @BeforeEach
  void setUp() {
    when(configurationStore.newConfiguration()).thenReturn(new ConfigurationData());
    underTest = new HuggingFaceProxyRepositoryApiRequestToConfigurationConverter(routingRuleStore);
    underTest.setConfigurationStore(configurationStore);
  }

  @Test
  void createsAHuggingFaceProxyWithAnUpstreamToken() throws Exception {
    HuggingFaceProxyRepositoryApiRequest request =
        new ObjectMapper().readValue(BODY, HuggingFaceProxyRepositoryApiRequest.class);

    Configuration configuration = underTest.convert(request);

    assertThat(configuration.getRepositoryName(), is("hf-proxy"));
    assertThat(configuration.getRecipeName(), is("huggingface-proxy"));
    assertThat(configuration.attributes("proxy").get("remoteUrl"), is("https://huggingface.co/"));
    assertThat(configuration.attributes("proxy").get("metadataMaxAge"), is(60));
    NestedAttributesMap authentication = configuration.attributes("httpclient").child("authentication");
    assertThat(authentication.get("type"), is("bearerToken"));
    assertThat(authentication.get("bearerTokenId"), is("hf_example"));
  }
}
