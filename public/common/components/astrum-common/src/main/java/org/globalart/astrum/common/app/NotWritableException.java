
package org.globalart.astrum.common.app;

/**
 * Thrown when the application is unable to accept writes.
 *
 * This may be because the application is frozen or the underlying storage is read-only.
 *
 * @since 3.21
 */
public class NotWritableException
    extends IllegalStateException
{
  private static final long serialVersionUID = 50364435356863049L;

  public NotWritableException(final String message) {
    super(message);
  }

  public NotWritableException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
