
package org.globalart.astrum.repository.security;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

import javax.annotation.Nullable;

/**
 * Request-scoped cache for content selector evaluation results.
 * <p>
 * This cache eliminates redundant selector evaluations within a single request,
 * particularly for Docker digest pulls from group repositories which can cause
 * N selectors × M member repos evaluations (ASTRUM-50181).
 * </p>
 * <p>
 * The cache is created per-request and should be passed through the security
 * check call chain. It is automatically cleared when the request completes.
 * </p>
 * <p>
 * Cache key format: userId:repositoryName:selectorName:path
 * </p>
 *
 * @since 3.x
 */
public class SelectorEvaluationCache
{
  private final Map<String, Boolean> cache = new ConcurrentHashMap<>();

  /**
   * Retrieves a cached selector evaluation result.
   *
   * @param key the cache key
   * @return the cached result, or null if not cached
   */
  @Nullable
  public Boolean get(final String key) {
    return cache.get(key);
  }

  /**
   * Stores a selector evaluation result in the cache.
   *
   * @param key the cache key
   * @param value the evaluation result
   */
  public void put(final String key, final Boolean value) {
    cache.put(key, value);
  }

  /**
   * Clears all cached entries.
   * Should be called when the request completes.
   */
  public void clear() {
    cache.clear();
  }

  /**
   * Returns the current number of cached entries.
   * Useful for testing and monitoring.
   *
   * @return cache size
   */
  public int size() {
    return cache.size();
  }
}
