
package org.globalart.astrum.repository.cache;

import java.io.Serializable;

/**
 * A key for the {@link NegativeCacheFacet} negative cache.
 *
 * @since 3.0
 */
public interface NegativeCacheKey
    extends Serializable
{
  /**
   * @param key child key
   * @return true if this key is a parent of passed in key.
   */
  boolean isParentOf(NegativeCacheKey key);
}
