
package org.globalart.astrum.crypto.secrets;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

/**
 * Storage for Astrum persisted secrets (e.g. passwords, access keys, etc.)
 */
public interface SecretsStore
{
  /**
   * Store a record of an encrypted secret. This store is not expected to encrypt the secret, it should be provided
   * in an encrypted form.
   *
   * @param purpose the purpose of the stored key, e.g. LDAP
   * @param keyId the identifier for the key (not the key itself) which was used to encrypt the secret
   * @param secret the previously encrypted secret to store
   * @param userId the user if known who initiated the request
   *
   * @return the id for the persisted record
   */
  int create(String purpose, @Nullable String keyId, String secret, @Nullable String userId);

  /**
   * Deletes a secret from the store by its identifier.
   *
   * @param id the internal identifier for the persisted secret
   * @return {@code true} if a record was removed, {@code false} otherwise.
   */
  boolean delete(int id);

  /**
   * Reads a secret from the store by its identifier.
   *
   * @param id the id of the secret
   *
   * @return the stored encrypted secret data
   */
  Optional<SecretData> read(int id);

  /**
   * Updates a record by its identifier with a new keyId and secret.
   *
   * @param id the identifier of the secret
   * @param oldSecret the encrypted secret to update, used to ensure the expected secret is being updated
   * @param keyId the identifier for the key (not the key itself) which was used to encrypt the secret
   * @param secret the previously encrypted secret to store
   * @return {@code true} if the record was updated, {@code false} otherwise.
   */
  boolean update(int id, String oldSecret, String keyId, String secret);

  /**
   * Check whether the store has records which were not encrypted by the provided key.
   *
   * @param keyId an identifier for a key used to encrypt secrets to check
   * @return true if there are records not encrypted with the provided key, false otherwise
   */
  boolean existWithDifferentKeyId(String keyId);

  /**
   * Get records stored in the store which were not encrypted by the provided key.
   *
   * @param keyId an identifier for a key used to encrypt secrets to filter results by
   * @param limit a limit to the number of records provided in the page
   * @return a list containing the page of records.
   */
  List<SecretData> fetchWithDifferentKeyId(String keyId, int limit);
}
