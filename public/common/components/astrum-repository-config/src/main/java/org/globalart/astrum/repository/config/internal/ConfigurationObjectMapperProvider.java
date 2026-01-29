
package org.globalart.astrum.repository.config.internal;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.config.ConfigurationObjectMapperCustomizer;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Provides {@link ObjectMapper} for repository configuration.
 *
 * @since 3.0
 */
@Component
@Qualifier(ConfigurationObjectMapperProvider.NAME)
public class ConfigurationObjectMapperProvider
    implements FactoryBean<ObjectMapper>
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String NAME = "repository-configuration";

  private final List<ConfigurationObjectMapperCustomizer> customizers;

  @Autowired
  public ConfigurationObjectMapperProvider(final List<ConfigurationObjectMapperCustomizer> customizersList) {
    this.customizers = checkNotNull(customizersList);
  }

  @Override
  public ObjectMapper getObject() throws Exception {
    ObjectMapper mapper = new ObjectMapper().enable(MapperFeature.ACCEPT_CASE_INSENSITIVE_ENUMS);
    mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
    for (ConfigurationObjectMapperCustomizer customizer : customizers) {
      customizer.customize(mapper);
    }
    // TODO: ISO-8601, joda
    // TODO: null handling
    return mapper;
  }

  @Override
  public Class<?> getObjectType() {
    return ObjectMapper.class;
  }
}
