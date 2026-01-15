
package org.globalart.astrum.siesta.internal;

import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.ext.Provider;

import org.globalart.astrum.rest.ExceptionMapperSupport;
import org.springframework.stereotype.Component;

/**
 * Unexpected generic {@link Throwable} exception mapper.
 *
 * This will handle all unexpected exceptions and override the default JAX-RS implementations behavior.
 *
 * @since 3.0
 */
@Component
@Provider
public class UnexpectedExceptionMapper
    extends ExceptionMapperSupport<Throwable>
{
  @Override
  protected Response convert(final Throwable exception, final String id) {
    return unexpectedResponse(exception, id);
  }
}
