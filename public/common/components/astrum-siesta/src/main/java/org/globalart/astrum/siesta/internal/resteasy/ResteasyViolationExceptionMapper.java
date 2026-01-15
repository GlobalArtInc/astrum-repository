
package org.globalart.astrum.siesta.internal.resteasy;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

import jakarta.ws.rs.core.Response.Status;
import jakarta.ws.rs.ext.Provider;

import org.globalart.astrum.rest.ValidationErrorXO;
import org.globalart.astrum.siesta.ValidationExceptionMapperSupport;

import org.jboss.resteasy.api.validation.ResteasyConstraintViolation;
import org.jboss.resteasy.api.validation.ResteasyViolationException;
import org.springframework.stereotype.Component;

import static org.jboss.resteasy.api.validation.ConstraintType.Type.RETURN_VALUE;

/**
 * Maps {@link ResteasyViolationException} to {@link Status#BAD_REQUEST} or {@link Status#INTERNAL_SERVER_ERROR}
 * in case of a violation on a methods return value, with a list of {@link ValidationErrorXO} as body.
 *
 * @since 3.0
 */
@Component
@Provider
public class ResteasyViolationExceptionMapper
    extends ValidationExceptionMapperSupport<ResteasyViolationException>
{
  @Override
  protected List<ValidationErrorXO> getValidationErrors(final ResteasyViolationException exception) {
    return getValidationErrors(exception.getViolations());
  }

  @Override
  protected Status getStatus(final ResteasyViolationException exception) {
    return getResponseStatus(exception.getViolations());
  }

  private List<ValidationErrorXO> getValidationErrors(final List<ResteasyConstraintViolation> violations) {
    final List<ValidationErrorXO> errors = new ArrayList<>();

    for (final ResteasyConstraintViolation violation : violations) {
      errors.add(new ValidationErrorXO(getPath(violation), violation.getMessage()));
    }

    return errors;
  }

  private Status getResponseStatus(final List<ResteasyConstraintViolation> violations) {
    final Iterator<ResteasyConstraintViolation> iterator = violations.iterator();

    if (iterator.hasNext()) {
      return getResponseStatus(iterator.next());
    }
    return Status.BAD_REQUEST;
  }

  private static Status getResponseStatus(final ResteasyConstraintViolation violation) {
    if (RETURN_VALUE.equals(violation.getConstraintType())) {
      return Status.INTERNAL_SERVER_ERROR;
    }
    return Status.BAD_REQUEST;
  }

  private static String getPath(final ResteasyConstraintViolation violation) {
    final String propertyPath = violation.getPath();
    final String propertyName = propertyPath.substring(propertyPath.lastIndexOf('.') + 1);

    return violation.type() + (!"".equals(propertyPath) ? ' ' + propertyName : "");
  }
}
