
package org.globalart.astrum.repository.cache;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Status;

/**
 * Negative cache management {@link Facet}.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface NegativeCacheFacet
    extends Facet
{
  /**
   * Retrieve an entry from negative cache.
   *
   * @param key cache key
   * @return cached {@link Status} or null if no cache entry found
   */
  @Nullable
  Status get(NegativeCacheKey key);

  /**
   * Add an entry to negative cache
   *
   * @param key cache key
   * @param status (404) status to be cached
   */
  void put(NegativeCacheKey key, Status status);

  /**
   * Removes an entry from negative cache.
   *
   * @param key cache key
   */
  void invalidate(NegativeCacheKey key);

  /**
   * Removes entry for passed in parent key and all is children (using
   * {@link NegativeCacheKey#isParentOf(NegativeCacheKey)}).
   *
   * @param key parent cache key
   */
  void invalidateSubset(NegativeCacheKey key);

  /**
   * Removes all entries from negative cache.
   */
  void invalidate();

  /**
   * Retrieves the cache key based on context.
   *
   * @param context view context
   * @return cache key
   */
  NegativeCacheKey getCacheKey(Context context);
}
