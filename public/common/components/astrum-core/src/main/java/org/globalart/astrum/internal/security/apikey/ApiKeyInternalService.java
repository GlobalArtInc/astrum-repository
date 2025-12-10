
package org.globalart.astrum.internal.security.apikey;

import org.globalart.astrum.security.authc.apikey.ApiKeyLowLevelService;

/**
 * Interface with internal use methods
 */
public interface ApiKeyInternalService
    extends ApiKeyLowLevelService
{
  /**
   * Purges any API-Keys associated with missing/deleted users.
   */
  int purgeApiKeys();
}
