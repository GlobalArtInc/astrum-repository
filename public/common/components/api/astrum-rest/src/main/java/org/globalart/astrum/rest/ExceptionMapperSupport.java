
package org.globalart.astrum.rest;

import java.util.UUID;

import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.ext.ExceptionMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.TEXT_PLAIN;

/**
 * Support for {@link ExceptionMapper} implementations.
 *
 * @since 3.0
 */
public abstract class ExceptionMapperSupport<E extends Throwable>
    implements ExceptionMapper<E>, Component
{
  public static final String X_SIESTA_FAULT_ID = "X-Siesta-FaultId";

  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Response toResponse(final E exception) {
    checkNotNull(exception);

    // Generate unique identifier
    final String id = generateFaultId();

    // debug/trace log exception details
    if (log.isTraceEnabled()) {
      log.trace("(ID {}) Mapping exception: " + exception, id, exception);
    }
    else if (log.isDebugEnabled()) {
      log.debug("(ID {}) Mapping exception: " + exception, id);
    }

    // Prepare the response
    Response response;
    try {
      response = convert(exception, id);
    }
    catch (Exception e) {
      log.warn("(ID {}) Failed to map exception", id, e);
      response = Response.serverError().entity(new FaultXO(id, e)).build();
    }

    // Add fault-id to the response as header
    response.getHeaders().putSingle(X_SIESTA_FAULT_ID, id);

    // Log terse (unless debug enabled) warning with fault details
    final Object entity = response.getEntity();
    log.warn("(ID {}) Response: [{}] {}; mapped from: {}",
        id,
        response.getStatus(),
        entity == null ? "(no entity/body)" : String.format("'%s'", entity),
        exception,
        log.isDebugEnabled() ? exception : null);

    return response;
  }

  private static String generateFaultId() {
    return UUID.randomUUID().toString();
  }

  /**
   * Convert the given exception into a response.
   *
   * @param exception The exception to convert.
   * @param id The unique identifier generated for this fault.
   */
  protected abstract Response convert(final E exception, final String id);

  protected Response unexpectedResponse(final Throwable exception, final String id) {
    // always log unexpected exception with stack
    log.warn("(ID {}) Unexpected exception: {}", id, exception.toString(), exception);

    return Response.serverError()
        .entity(String.format("ERROR: (ID %s) %s", id, exception))
        .type(TEXT_PLAIN)
        .build();
  }
}
