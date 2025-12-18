
package org.globalart.astrum.security.jwt;

import java.util.Optional;

/**
 * Store for accessing the JWT secret.
 *
 * @since 3.38
 */
public interface SecretStore
{
  /**
   * Retrieve the JWT secret.
   *
   * @return the secret if it exists otherwise {@link Optional#empty}.
   */
  Optional<String> getSecret();

  /**
   * Set the JWT secret.
   */
  void setSecret(final String secret);

  /**
   * Generate the new JWT secret by using the UUID.
   */
  void generateNewSecret();
}
