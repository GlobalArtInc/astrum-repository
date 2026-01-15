
package org.globalart.astrum.siesta.internal;

import java.util.Map;
import java.util.UUID;

import org.globalart.astrum.rest.ExceptionMapperSupport;

import com.google.common.annotations.VisibleForTesting;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.ext.ExceptionMapper;
import jakarta.ws.rs.ext.Provider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Standard {@link WebApplicationException} exception mapper.
 *
 * This is needed to restore default response behavior when {@link UnexpectedExceptionMapper} is installed.
 *
 * @since 3.0
 */
@Component
@Provider
public class WebappExceptionMapper
    implements ExceptionMapper<WebApplicationException>, org.globalart.astrum.rest.Component
{
  private static final Logger log = LoggerFactory.getLogger(WebappExceptionMapper.class);

  static final String STATUS_MESSAGE = "status-message";

  static final String STATUS_CODE = "status-code";

  static final String SIESTA_FAULTID = "siesta-faultid";

  @VisibleForTesting
  Response convert(final WebApplicationException exception, final String id) {
    // build new response to avoid potential information disclosure (CVE-2020-25633)
    Response response = exception.getResponse();
    Object entity = response.getEntity();
    log.debug("(ID {}) Response: [{}], entity: {}",
        id, response.getStatus(), entity == null ? "(no entity/body)" : String.format("'%s'", entity), exception);
    return Response.status(response.getStatus())
        .entity(Map.of(SIESTA_FAULTID, id, STATUS_CODE, response.getStatus(), STATUS_MESSAGE,
            response.getStatusInfo()))
        .type(MediaType.APPLICATION_JSON)
        // Add fault-id to the response as header
        .header(ExceptionMapperSupport.X_SIESTA_FAULT_ID, id)
        .build();
  }

  @Override
  public Response toResponse(final WebApplicationException exception) {
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

    return convert(exception, id);
  }

  private static String generateFaultId() {
    return UUID.randomUUID().toString();
  }
}
