
package org.globalart.astrum.internal.httpclient;

import java.util.Map;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.httpclient.config.ConnectionConfiguration;
import org.globalart.astrum.kv.KeyValueStore;
import org.globalart.astrum.repository.httpclient.HttpClientConfig;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertEquals;

/**
 * @since 3.1.0
 */
@RunWith(MockitoJUnitRunner.class)
public class HttpClientConfigurationObjectMapperCustomizerTest
{

  @Mock
  private SecretsService secretsService;

  @Mock
  private KeyValueStore keyValueStore;

  private HttpClientConfigurationObjectMapperCustomizer customizer;

  private ObjectMapper objectMapper;

  @Before
  public void setup() {
    customizer = new HttpClientConfigurationObjectMapperCustomizer(secretsService, keyValueStore);
    objectMapper = new ObjectMapper();
    customizer.customize(objectMapper);
  }

  @Test
  public void canSerializeAndDeserializeHttpClientConfigurationTimeouts() throws Exception {
    // Given: A configuration with a timeout set
    HttpClientConfig config = new HttpClientConfig();
    config.connection = new ConnectionConfiguration();
    config.connection.setTimeout(Time.seconds(1));

    // When: Serializing that with ObjectMapper
    String json = objectMapper.writeValueAsString(config);
    Map<String, Object> map = new ObjectMapper().readValue(json, Map.class);
    Map<String, Object> innermap = (Map<String, Object>) map.get("connection");
    // Then: We can confirm that the timeout is set to a number
    assertEquals(1, innermap.get("timeout"));

    // When: We convert back to an object with ObjectMapper
    HttpClientConfig marshalledConfig = objectMapper.readValue(json, HttpClientConfig.class);

    // Then: The time is correctly converted
    assertEquals(Time.seconds(1), marshalledConfig.connection.getTimeout());
  }
}
