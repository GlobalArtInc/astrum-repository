
package org.globalart.astrum.crypto.internal.error;

/**
 * Exception class to identify all the failures related to encryption/decryption
 */
public class CipherException
    extends RuntimeException
{
  public CipherException(final String message) {
    super(message);
  }

  public CipherException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
