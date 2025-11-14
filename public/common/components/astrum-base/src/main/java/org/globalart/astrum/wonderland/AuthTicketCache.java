
package org.globalart.astrum.wonderland;

/**
 * Manages cache (and expiration) of authentication tickets.
 *
 * @since 2.7
 */
public interface AuthTicketCache
{
  String EXPIRE_VALUE = "${wonderland.authTicketCache.expireAfter:20s}";

  /**
   * Add token to the cache.
   */
  void add(String user, String token, String realmName);

  /**
   * Remove token from cache.
   *
   * @return True if the token existed (was added and not yet expired)
   */
  boolean remove(String user, String token, String realmName);
}
