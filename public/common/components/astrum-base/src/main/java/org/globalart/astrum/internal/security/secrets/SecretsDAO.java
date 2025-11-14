
package org.globalart.astrum.internal.security.secrets;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.crypto.secrets.SecretData;
import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

public interface SecretsDAO
    extends DataAccess
{
  /**
   * Store a record of an encrypted secret. This store is not expected to encrypt the secret, it should be provided
   * in an encrypted form.
   *
   * @param secretData the previously encrypted secret data to store
   * @return the id for the persisted record
   */
  int create(@Param("secretData") SecretData secretData);

  /**
   * Deletes a secret from the store by its identifier.
   *
   * @param id the internal identifier for the persisted secret
   * @return {@code true} if a record was removed, {@code false} otherwise.
   */
  int delete(@Param("id") int id);

  /**
   * Reads a secret from the store by its identifier.
   *
   * @param id the id of the secret
   *
   * @return the stored encrypted secret data
   */
  Optional<SecretData> read(@Param("id") int id);

  /**
   * Updates a record by its identifier with a new keyId and secret.
   *
   * @param id the identifier of the secret
   * @param oldSecret the current encrypted secret to update, used to ensure the expected secret is being updated
   * @param keyId the identifier for the key (not the key itself) which was used to encrypt the secret
   * @param secret the previously encrypted secret to store
   * @return the id for the updated record
   */
  int update(
      @Param("id") int id,
      @Param("oldSecret") String oldSecret,
      @Param("keyId") String keyId,
      @Param("secret") String secret);

  /**
   * Check whether the store has records which were not encrypted by the provided key.
   *
   * @param keyId an identifier for a key used to encrypt secrets to check
   * @return true if there are records not encrypted with the provided key, false otherwise
   */
  boolean existWithDifferentKeyId(@Param("keyId") String keyId);

  /**
   * Get records stored in the store which were not encrypted by the provided key.
   *
   * @param keyId an identifier for a key used to encrypt secrets to filter results by
   * @param limit a limit to the number of records provided in the page
   * @return a list containing the page of records.
   */
  List<SecretData> fetchWithDifferentKeyId(
      @Param("keyId") String keyId,
      @Param("limit") int limit);
}
