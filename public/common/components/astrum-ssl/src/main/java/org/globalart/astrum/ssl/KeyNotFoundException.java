
package org.globalart.astrum.ssl;

/**
 * Exception indicating that the private key you tried to retrieve does not exist.
 *
 * @since 3.0
 */
public class KeyNotFoundException
    extends KeystoreException
{
  public KeyNotFoundException(String message) {
    super(message);
  }

  public KeyNotFoundException(String message, Throwable cause) {
    super(message, cause);
  }
}
