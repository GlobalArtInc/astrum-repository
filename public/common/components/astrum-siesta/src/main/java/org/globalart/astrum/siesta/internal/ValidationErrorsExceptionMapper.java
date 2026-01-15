
package org.globalart.astrum.siesta.internal;

import java.util.List;

import jakarta.ws.rs.ext.Provider;

import org.globalart.astrum.rest.ValidationErrorXO;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.siesta.ValidationExceptionMapperSupport;
import org.springframework.stereotype.Component;

/**
 * Maps {@link ValidationErrorsException} to 400 with a list of {@link ValidationErrorXO} as body.
 *
 * @since 3.0
 */
@Component
@Provider
public class ValidationErrorsExceptionMapper
    extends ValidationExceptionMapperSupport<ValidationErrorsException>
{
  @Override
  protected List<ValidationErrorXO> getValidationErrors(final ValidationErrorsException exception) {
    return exception.getValidationErrors();
  }
}
