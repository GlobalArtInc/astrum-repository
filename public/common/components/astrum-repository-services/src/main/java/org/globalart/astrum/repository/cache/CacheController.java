
package org.globalart.astrum.repository.cache;

import javax.annotation.Nullable;

import com.google.common.annotations.VisibleForTesting;
import org.joda.time.DateTime;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * A support class which implements basic cache-control logic.
 *
 * @since 3.0
 */
public class CacheController
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static String newCacheToken() {
    return Long.toString(System.nanoTime());
  }

  private final int contentMaxAgeSeconds;

  private volatile String cacheToken;

  public CacheController(final int contentMaxAgeSeconds, @Nullable final String cacheToken) {
    this.contentMaxAgeSeconds = contentMaxAgeSeconds;
    this.cacheToken = cacheToken;
  }

  /**
   * After invoking this method, all {@link #isStale(CacheInfo)} checks will return true that has {@link CacheInfo} not
   * carrying same token as created in this method.
   */
  public void invalidateCache() {
    this.cacheToken = newCacheToken();
  }

  public void setCache(final String token) {
    this.cacheToken = token;
  }

  /**
   * Returns the currently effective {@link CacheInfo} with "now" timestamp.
   */
  public CacheInfo current() {
    return new CacheInfo(DateTime.now(), cacheToken);
  }

  /**
   * Returns {@code true} if passed in cache info carries stale information, detected either by cache token or
   * age of the info.
   */
  public boolean isStale(final CacheInfo cacheInfo) {
    if (cacheInfo.isInvalidated()) {
      log.debug("Content invalidated");
      return true;
    }
    if (cacheToken != null && !cacheToken.equals(cacheInfo.getCacheToken())) {
      log.debug("Content expired (cacheToken)");
      return true;
    }
    if (contentMaxAgeSeconds < 0) {
      log.trace("Content max age checking disabled");
      return false;
    }
    if (cacheInfo.getLastVerified().isBefore(new DateTime().minusSeconds(contentMaxAgeSeconds))) {
      log.debug("Content expired (age)");
      return true;
    }
    return false;
  }

  @VisibleForTesting
  public int getContentMaxAgeSeconds() {
    return contentMaxAgeSeconds;
  }
}
