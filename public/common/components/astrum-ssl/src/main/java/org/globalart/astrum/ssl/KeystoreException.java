
package org.globalart.astrum.ssl;

/**
 * Exception indicating that a problem occurred accessing the keystore.
 *
 * @since 3.0
 */
public class KeystoreException
    extends Exception
{
  public KeystoreException(String message) {
    super(message);
  }

  public KeystoreException(String message, Throwable cause) {
    super(message, cause);
  }
}
