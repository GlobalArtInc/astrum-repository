
package org.globalart.astrum.internal.httpclient.handlers;

import java.util.function.Supplier;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretDeserializer;
import org.globalart.astrum.crypto.secrets.SecretsFactory;
import org.globalart.astrum.datastore.mybatis.AbstractJsonTypeHandler;
import org.globalart.astrum.datastore.mybatis.OverrideIgnoreTypeIntrospector;
import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.internal.httpclient.AuthenticationConfigurationDeserializer;
import org.globalart.astrum.internal.httpclient.AuthenticationConfigurationSerializer;
import org.globalart.astrum.internal.httpclient.SecondsDeserializer;
import org.globalart.astrum.internal.httpclient.SecondsSerializer;
import org.globalart.astrum.kv.KeyValueStore;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.module.SimpleModule;
import com.google.common.collect.ImmutableList;
import org.apache.ibatis.type.TypeHandler;

/**
 * MyBatis {@link TypeHandler} that sets up an {@link com.fasterxml.jackson.databind.ObjectMapper} for
 * correct encryption-at-rest of {@link org.globalart.astrum.httpclient.config.HttpClientConfiguration}s.
 *
 * @since 3.21
 */
public abstract class HttpClientConfigurationHandler<T>
    extends AbstractJsonTypeHandler<T>
{
  // this guarantees the constructor and buildObjectMapper work on the same mapper, regardless which runs first
  private static final ThreadLocal<ObjectMapper> constructingMapper = ThreadLocal.withInitial(ObjectMapper::new);

  protected HttpClientConfigurationHandler(final SecretsFactory secretsFactory, final KeyValueStore keyValueStore) {
    constructingMapper.get()
        .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
        .setAnnotationIntrospector(new OverrideIgnoreTypeIntrospector(ImmutableList.of(Secret.class)))
        // register custom serializers and deserializers
        // - goodies Time is our internal Time representation
        // - AuthenticationConfiguration needs a tiny bit of logic for resolving the proper impl and encryption
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
                .addDeserializer(Secret.class, new SecretDeserializer(secretsFactory))

        );
  }

  @Override
  protected ObjectMapper buildObjectMapper(final Supplier<ObjectMapper> mapperFactory) {
    // use our custom mapper which re-uses existing serialization/deserialization helpers
    return constructingMapper.get();
  }
}
