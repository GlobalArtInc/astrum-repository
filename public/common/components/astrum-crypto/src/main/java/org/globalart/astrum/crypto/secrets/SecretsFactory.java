
package org.globalart.astrum.crypto.secrets;

/**
 * Provides {@link Secret} instances which are backed by services for retrieval and decryption of secrets by their id.
 */
public interface SecretsFactory
{
  /**
   * <p>
   * Create a {@link Secret} which can be used to retrieve and decrypt a persisted secret by id. For legacy reasons
   * this may also be a persisted secret.
   * </p>
   *
   * <p>
   * Existence of the provided token will not be verified until decryption
   * </p>
   *
   * @param token an identifier previously provided by {@link SecretsService} or a legacy encrypted secret.
   *
   * @return a secret for the provided value.
   */
  Secret from(String token);
}
