
package org.globalart.astrum.rest;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Thrown when there are request validation errors.
 *
 * @see ValidationErrorXO
 * @since 3.0
 */
public class ValidationErrorsException
    extends RuntimeException
{
  private final List<ValidationErrorXO> errors = new ArrayList<ValidationErrorXO>();

  public ValidationErrorsException() {
    super();
  }

  public ValidationErrorsException(final String message) {
    errors.add(new ValidationErrorXO(message));
  }

  public ValidationErrorsException(final String message, final Throwable e) {
    super(e);
    errors.add(new ValidationErrorXO(message));
  }

  public ValidationErrorsException(final String id, final String message) {
    errors.add(new ValidationErrorXO(id, message));
  }

  public ValidationErrorsException withError(final String message) {
    errors.add(new ValidationErrorXO(message));
    return this;
  }

  public ValidationErrorsException withError(final String id, final String message) {
    errors.add(new ValidationErrorXO(id, message));
    return this;
  }

  public ValidationErrorsException withErrors(final ValidationErrorXO... validationErrors) {
    checkNotNull(validationErrors);
    errors.addAll(Arrays.asList(validationErrors));
    return this;
  }

  public ValidationErrorsException withErrors(final List<ValidationErrorXO> validationErrors) {
    checkNotNull(validationErrors);
    errors.addAll(validationErrors);
    return this;
  }

  public List<ValidationErrorXO> getValidationErrors() {
    return errors;
  }

  public boolean hasValidationErrors() {
    return !errors.isEmpty();
  }

  @Override
  public String getMessage() {
    final StringBuilder sb = new StringBuilder();
    for (final ValidationErrorXO error : errors) {
      if (sb.length() > 0) {
        sb.append(", ");
      }
      sb.append(error.getMessage());
    }
    return sb.length() == 0 ? "(No validation errors)" : sb.toString();
  }
}
