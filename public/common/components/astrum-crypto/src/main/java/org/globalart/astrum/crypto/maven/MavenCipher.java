
package org.globalart.astrum.crypto.maven;

/**
 * Cipher that uses Apache Maven format (aka Plexus Cipher).
 *
 * Is meant to be a drop-in replacement for plexus-cipher.
 * 
 * @since 3.0
 */
public interface MavenCipher
{
  String encrypt(CharSequence str, String passPhrase);

  String decrypt(String str, String passPhrase);

  boolean isPasswordCipher(CharSequence str);

  /**
   * @since 3.21
   */
  char[] decryptChars(String str, String passPhrase);
}
