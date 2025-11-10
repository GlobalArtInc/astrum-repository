
package org.globalart.astrum.crypto.secrets.internal;

import java.util.Optional;

import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.FixedEncryption;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.SecretEncryptionKey;

/**
 * Simple interface representing a source of secret keys to be used later on secrets encryption
 */
public interface EncryptionKeySource
{
  /**
   * Retrieves the currently configured key to use for encrypting secrets
   *
   * @return an {@link Optional<SecretEncryptionKey>} filled if there is a key being used at the moment
   */
  Optional<SecretEncryptionKey> getActiveKey();

  /**
   * Retrieves the currently configured key to use for encrypting principals
   *
   * @return an {@link Optional<SecretEncryptionKey>} filled if there is a key being used at the moment
   */
  Optional<FixedEncryption> getFixedEncryption();

  /**
   * Retrieves the previously configured fixed encryption configuration, if available.
   *
   * @return an {@link Optional<FixedEncryption>} containing the previous fixed encryption configuration,,
   *         or an empty {@link Optional} if no previous configuration is available.
   */
  Optional<FixedEncryption> getPreviousFixedEncryption();

  /**
   * Retrieve an encryption key by its identifier
   *
   * @param keyId the id of the secret key to be retrieved
   * @return an {@link Optional} with a {@link SecretEncryptionKey} if the secret key was found
   */
  Optional<SecretEncryptionKey> getKey(String keyId);

  /**
   * Sets the active key to be used for encrypting secrets
   *
   * @param keyId the id of the secret to be used
   */
  void setActiveKey(String keyId);

}
