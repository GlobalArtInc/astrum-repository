
package org.globalart.astrum.extdirect.model;

import jakarta.validation.ConstraintViolationException;

/**
 * Ext.Direct response builder.
 *
 * @since 3.0
 */
public class Responses
{
  private Responses() {
    // empty
  }

  public static Response<Object> success() {
    return success(null);
  }

  public static <T> Response<T> success(T data) {
    return new Response<>(true, data);
  }

  public static ErrorResponse error(final Throwable cause) {
    return new ErrorResponse(cause);
  }

  public static ErrorResponse error(final String message) {
    return new ErrorResponse(message);
  }

  public static ValidationResponse invalid(final ConstraintViolationException cause) {
    return new ValidationResponse(cause);
  }
}
