
package org.globalart.astrum.common.app;

/**
 * Thrown when the application is unable to accept reads.
 *
 * This may be because the underlying storage is not accessible.
 *
 * @since 3.21
 */
public class NotReadableException
    extends IllegalStateException
{
  private static final long serialVersionUID = 3425411938965871948L;

  public NotReadableException(final String message) {
    super(message);
  }

  public NotReadableException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
