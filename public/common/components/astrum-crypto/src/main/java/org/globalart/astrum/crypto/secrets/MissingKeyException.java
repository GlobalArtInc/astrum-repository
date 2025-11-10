
package org.globalart.astrum.crypto.secrets;

/**
 * Thrown when the encryption key does not exist.
 */
public class MissingKeyException
    extends RuntimeException
{
  public MissingKeyException(final String message) {
    super(message);
  }

  public MissingKeyException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
