
package org.globalart.astrum.crypto.secrets;

/**
 * Intended to hold an unencrypted secret in memory for passing around in the application.
 *
 * Use this in a scenario where you have a class which expects a Secret but that secret should not be stored in the
 * database.
 */
public class DecryptedSecret
    implements Secret
{
  private final char[] secret;

  public DecryptedSecret(final char[] secret) {
    this.secret = secret;
  }

  public String getId() {
    throw new UnsupportedOperationException();
  }

  public char[] decrypt() {
    return secret;
  }
}
