
package org.globalart.astrum.repository;

/**
 * Content is of an incorrect or indeterminate type.
 *
 * @since 3.0
 */
public class InvalidContentException
    extends RuntimeException
{
  public InvalidContentException(final String message) {
    super(message);
  }

  public InvalidContentException(final String message, final Throwable cause) {
    super(message, cause);
  }

  public InvalidContentException(final Throwable cause) {
    super(cause);
  }
}
