
package org.globalart.astrum.repository;

/**
 * Thrown when an operation cannot complete because a concurrent operation has modified shared state.
 * Maps to HTTP 409 Conflict.
 */
public class ConcurrentOperationException
    extends RuntimeException
{
  private static final long serialVersionUID = 1L;

  public ConcurrentOperationException(final String message) {
    super(message);
  }

  public ConcurrentOperationException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
