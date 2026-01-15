
package org.globalart.astrum.siesta.internal;

import java.util.List;
import java.util.stream.Collectors;

import jakarta.ws.rs.core.GenericEntity;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.ExceptionMapperSupport;
import org.globalart.astrum.rest.ValidationErrorXO;

import com.fasterxml.jackson.databind.JsonMappingException;
import org.springframework.stereotype.Component;

/**
 * Maps {@link JsonMappingExceptionMapper} to {@link Status#BAD_REQUEST} in case there is a deserialization error.
 *
 * @since 3.19
 */
@Component
public class JsonMappingExceptionMapper
    extends ExceptionMapperSupport<JsonMappingException>
{
  @Override
  protected Response convert(final JsonMappingException exception, final String id) {
    final List<ValidationErrorXO> errors = exception.getPath()
        .stream()
        .map(reference -> new ValidationErrorXO(reference.getFieldName(), exception.getOriginalMessage()))
        .collect(Collectors.toList());

    return Response.status(Status.BAD_REQUEST)
        .entity(new GenericEntity<List<ValidationErrorXO>>(errors)
        {
          @Override
          public String toString() {
            return getEntity().toString();
          }
        })
        .type(MediaType.APPLICATION_JSON)
        .build();
  }
}
