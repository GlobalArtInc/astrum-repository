
package org.globalart.astrum.crypto.secrets;

/**
 * Thrown when the re-encryption is not supported e.g. database is not ready to support re-encryption.
 */
public class ReEncryptionNotSupportedException
    extends RuntimeException
{
  public ReEncryptionNotSupportedException(final String message) {
    super(message);
  }

}
