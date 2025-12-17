
package org.globalart.astrum.security.authc.apikey;

import java.time.OffsetDateTime;
import java.util.Optional;

import org.apache.shiro.subject.PrincipalCollection;

/**
 * Persistent mapping between principals (such as user IDs) and API-Keys.
 *
 * @since 3.0
 */
public interface ApiKeyService
{
  /**
   * Creates an API-Key and assigns it to the given principals in given domain.
   */
  char[] createApiKey(String domain, PrincipalCollection principals);

  /**
   * Gets the current API-Key assigned to the given principals in given domain.
   *
   * @return {@code null} if no key has been assigned
   */
  Optional<ApiKey> getApiKey(String domain, PrincipalCollection principals);

  /**
   * Retrieves the principals associated with the given API-Key in given domain.
   *
   * @return {@code null} if the key is invalid or stale
   */
  Optional<ApiKey> getApiKeyByToken(String domain, char[] apiKey);

  /**
   * Count all the keys for the provided domain
   */
  int count(String domain);

  /**
   * Deletes the API-Key associated with the given principals in given domain.
   */
  int deleteApiKey(String domain, PrincipalCollection principals);

  /**
   * Deletes every API-Key associated with the given principals in every domain.
   */
  int deleteApiKeys(PrincipalCollection principals);

  /**
   * Deletes all API-Keys for the specified domain
   */
  int deleteApiKeys(String domain);

  /**
   * Remove all expired API-Keys
   */
  int deleteApiKeys(OffsetDateTime expiration);

  /**
   * Remove all expired API-Keys for the specified domain
   */
  int deleteApiKeys(String domain, OffsetDateTime expiration);
}
