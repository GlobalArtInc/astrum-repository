
package org.globalart.astrum.crypto.secrets;

import java.util.Optional;

/**
 * Validator of encryption keys, this is useful to check if a key is accessible before using it or get the
 * active key id
 */
public interface EncryptionKeyValidator
{
  /**
   * Checks the provided key is accessible
   *
   * @param keyId the key to check
   * @return {@code true} if the key is accessible, {@code false} otherwise
   */
  boolean isValidKey(String keyId);

  /**
   * Gets the active key id , if present
   *
   * @return an {@link Optional} with the key id String if found
   */
  Optional<String> getActiveKeyId();
}
