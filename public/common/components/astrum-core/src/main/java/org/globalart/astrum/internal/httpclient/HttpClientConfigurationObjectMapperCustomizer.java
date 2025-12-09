
package org.globalart.astrum.internal.httpclient;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretDeserializer;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.datastore.mybatis.OverrideIgnoreTypeIntrospector;
import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.kv.KeyValueStore;
import org.globalart.astrum.repository.config.ConfigurationObjectMapperCustomizer;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.module.SimpleModule;
import com.google.common.collect.ImmutableList;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * HTTP-client specific {@link ConfigurationObjectMapperCustomizer} that registers custom deserializer
 * with {@link ObjectMapper}.
 *
 * @see AuthenticationConfigurationDeserializer
 */
@Component
public class HttpClientConfigurationObjectMapperCustomizer
    implements ConfigurationObjectMapperCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final SecretsService secretsService;

  private final KeyValueStore keyValueStore;

  @Autowired
  public HttpClientConfigurationObjectMapperCustomizer(
      final SecretsService secretsService,
      final KeyValueStore keyValueStore)
  {
    this.secretsService = checkNotNull(secretsService);
    this.keyValueStore = checkNotNull(keyValueStore);
  }

  @Override
  public void customize(final ObjectMapper objectMapper) {
    objectMapper
        .setAnnotationIntrospector(new OverrideIgnoreTypeIntrospector(ImmutableList.of(Secret.class)))
        .registerModule(
            new SimpleModule()
                .addSerializer(
                    Time.class,
                    new SecondsSerializer())
                .addDeserializer(
                    Time.class,
                    new SecondsDeserializer())
                .addSerializer(
                    AuthenticationConfiguration.class,
                    new AuthenticationConfigurationSerializer(keyValueStore))
                .addDeserializer(
                    AuthenticationConfiguration.class,
                    new AuthenticationConfigurationDeserializer())
                .addDeserializer(Secret.class, new SecretDeserializer(secretsService)));
  }
}
