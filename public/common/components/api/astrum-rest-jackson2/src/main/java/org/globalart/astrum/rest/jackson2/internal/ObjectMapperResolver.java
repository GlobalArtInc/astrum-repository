
package org.globalart.astrum.rest.jackson2.internal;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.ext.ContextResolver;
import jakarta.ws.rs.ext.Provider;

import org.globalart.astrum.rest.Component;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.springframework.beans.factory.annotation.Qualifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Jackson {@link ObjectMapper} resolver.
 *
 * This will use the mapper bound to the name "siesta".
 *
 * @since 3.0
 * @see ObjectMapperProvider
 */
@org.springframework.stereotype.Component
@Provider
@Produces(MediaType.APPLICATION_JSON)
public class ObjectMapperResolver
    implements ContextResolver<ObjectMapper>, Component
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final jakarta.inject.Provider<ObjectMapper> mapperProvider;

  @Autowired
  public ObjectMapperResolver(@Qualifier("siesta") final jakarta.inject.Provider<ObjectMapper> mapperProvider) {
    this.mapperProvider = checkNotNull(mapperProvider);
  }

  @Override
  public ObjectMapper getContext(final Class<?> type) {
    return mapperProvider.get();
  }
}
