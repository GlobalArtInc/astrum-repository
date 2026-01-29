
package org.globalart.astrum.repository;

/**
 * Thrown when an illegal operation has been encountered.
 *
 * @since 3.0
 */
public class IllegalOperationException
    extends RuntimeException
{
  public IllegalOperationException(final String message) {
    super(message);
  }

  public IllegalOperationException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
