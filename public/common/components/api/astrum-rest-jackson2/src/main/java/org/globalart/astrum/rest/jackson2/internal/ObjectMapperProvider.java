
package org.globalart.astrum.rest.jackson2.internal;

import java.util.List;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.collect.json.NestedAttributesMapDeserializer;
import org.globalart.astrum.common.collect.json.NestedAttributesMapSerializer;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.Module;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.databind.ser.impl.SimpleFilterProvider;
import com.fasterxml.jackson.databind.module.SimpleModule;
import com.fasterxml.jackson.datatype.joda.JodaModule;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import jakarta.inject.Provider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Jackson {@link ObjectMapper} provider for use with Siesta.
 *
 * @since 3.0
 */
@Component
@Qualifier("siesta")
public class ObjectMapperProvider
    implements Provider<ObjectMapper>, FactoryBean<ObjectMapper>
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ObjectMapper mapper;

  public ObjectMapperProvider(final List<Module> modules) {
    this.mapper = new ObjectMapper()
        .enable(SerializationFeature.INDENT_OUTPUT)
        .enable(MapperFeature.ACCEPT_CASE_INSENSITIVE_ENUMS)
        .disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS)
        .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
        .setFilterProvider(new SimpleFilterProvider().setFailOnUnknownId(false));
    mapper.registerModule(new JavaTimeModule());
    mapper.registerModule(new JodaModule());
    mapper.registerModule(new SimpleModule()
        .addSerializer(NestedAttributesMap.class, new NestedAttributesMapSerializer())
        .addDeserializer(NestedAttributesMap.class, new NestedAttributesMapDeserializer()));
    modules.forEach(mapper::registerModule);

  }

  @Override
  public ObjectMapper get() {
    return mapper;
  }

  @Override
  public ObjectMapper getObject() throws Exception {
    return mapper;
  }

  @Override
  public Class<?> getObjectType() {
    return ObjectMapper.class;
  }
}
