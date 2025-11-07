
package org.globalart.astrum.crypto.internal;

import org.globalart.astrum.crypto.internal.error.CipherException;
import org.globalart.astrum.crypto.secrets.EncryptedSecret;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.SecretEncryptionKey;

/**
 * Factory for {@link PbeCipher}s, for secrets encryption stored in the PHC string format.
 * <p>
 * for further info check
 * <a href="https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md">PHC String format</a>
 */
public interface PbeCipherFactory
{
  interface PbeCipher
  {
    EncryptedSecret encrypt(final byte[] bytes) throws CipherException;

    byte[] decrypt() throws CipherException;

    byte[] decrypt(byte[] value) throws CipherException;

    boolean isDefaultCipher();
  }

  // Create a PBE cipher using an algorithm from ASTRUM_SECRETS_ALGORITHM property(e.g., PBKDF2 with SHA-256 or SHA-1),
  // needed for encryption
  PbeCipher create(SecretEncryptionKey secretEncryptionKey) throws CipherException;

  // Create a PBE cipher using the specified algorithm from encryptedSecret, needed for decryption and verify
  PbeCipher create(SecretEncryptionKey secretEncryptionKey, String encryptedSecret) throws CipherException;

  // create a PBE cipher using an algorithm from ASTRUM_SECRETS_ALGORITHM property, salt, IV and iterations
  PbeCipher create(
      SecretEncryptionKey secretEncryptionKey,
      String salt,
      String iv,
      Integer iterations) throws CipherException;
}
