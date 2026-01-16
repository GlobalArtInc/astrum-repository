
package org.globalart.astrum.siesta;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.MediaType.APPLICATION_XML;
import org.springframework.stereotype.Component;

@Component
@Path("/validationErrors")
public class ValidationErrorsResource
    implements Resource
{
  private final Logger log = LoggerFactory.getLogger(getClass());

  @PUT
  @Path("/manual/multiple")
  @Consumes({APPLICATION_XML, APPLICATION_JSON})
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public UserXO putWithMultipleManualValidations(final UserXO user) {
    log.info("PUT user: {}", user);

    final ValidationErrorsException validationErrors = new ValidationErrorsException();
    if (user.getName() == null) {
      validationErrors.withError("name", "Name cannot be null");
    }
    if (user.getDescription() == null) {
      validationErrors.withError("description", "Description cannot be null");
    }

    if (validationErrors.hasValidationErrors()) {
      throw validationErrors;
    }

    return user;
  }

  @PUT
  @Path("/manual/single")
  @Consumes({APPLICATION_XML, APPLICATION_JSON})
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public UserXO putWithSingleManualValidation(final UserXO user) {
    log.info("PUT user: {}", user);

    if (user.getName() == null) {
      throw new ValidationErrorsException("name", "Name cannot be null");
    }
    if (user.getDescription() == null) {
      throw new ValidationErrorsException("description", "Description cannot be null");
    }

    return user;
  }
}
