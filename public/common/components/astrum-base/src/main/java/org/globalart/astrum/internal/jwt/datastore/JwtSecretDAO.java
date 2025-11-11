
package org.globalart.astrum.internal.jwt.datastore;

import org.globalart.astrum.datastore.api.SingletonDataAccess;

/**
 * DAO for accessing the JWT secret
 *
 * @since 3.38
 */
public interface JwtSecretDAO
    extends SingletonDataAccess<String>
{
  /**
   * The new secret will be set up only if it is not presented yet.
   *
   * @param secret the new JWT secret.
   */
  void setIfEmpty(String secret);
}
