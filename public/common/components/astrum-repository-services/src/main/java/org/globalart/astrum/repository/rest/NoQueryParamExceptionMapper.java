
package org.globalart.astrum.repository.rest;

import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.ExceptionMapperSupport;

import static org.globalart.astrum.rest.SimpleApiResponse.badRequest;
import org.springframework.stereotype.Component;

/**
 * @since 3.14
 */
@Component
public class NoQueryParamExceptionMapper
    extends ExceptionMapperSupport<NoQueryParamException>
{
  @Override
  protected Response convert(final NoQueryParamException exception, final String id) {
    return badRequest(exception.getMessage());
  }
}
