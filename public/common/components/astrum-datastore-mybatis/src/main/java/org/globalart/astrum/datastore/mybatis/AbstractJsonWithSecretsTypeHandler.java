
package org.globalart.astrum.datastore.mybatis;

import java.util.function.Supplier;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretDeserializer;
import org.globalart.astrum.crypto.secrets.SecretSerializer;
import org.globalart.astrum.crypto.secrets.SecretsFactory;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.module.SimpleModule;
import com.fasterxml.jackson.datatype.jdk8.Jdk8Module;
import com.google.common.collect.ImmutableList;
import org.apache.ibatis.type.TypeHandler;

/**
 * MyBatis {@link TypeHandler} that sets up an {@link ObjectMapper} with some custom serializers to be able to
 * store/retrieve object attributes with type {@link Secret}
 *
 * @param <T> the java type of the object to be handled
 */
public class AbstractJsonWithSecretsTypeHandler<T>
    extends AbstractJsonTypeHandler<T>
{
  // this guarantees the constructor and buildObjectMapper work on the same mapper, regardless which runs first
  private static final ThreadLocal<ObjectMapper> mapper = ThreadLocal.withInitial(ObjectMapper::new);

  protected AbstractJsonWithSecretsTypeHandler(final SecretsFactory secretsFactory) {
    mapper.get()
        .setAnnotationIntrospector(new OverrideIgnoreTypeIntrospector(
            ImmutableList.of(Secret.class)))
        .registerModule(new Jdk8Module())
        .registerModule(new SimpleModule()
            .addSerializer(
                Secret.class,
                new SecretSerializer())
            .addDeserializer(
                Secret.class,
                new SecretDeserializer(secretsFactory)));
  }

  @Override
  protected ObjectMapper buildObjectMapper(final Supplier<ObjectMapper> mapperFactory) {
    return mapper.get();
  }
}
