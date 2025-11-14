
package org.globalart.astrum.internal.wonderland;

import java.time.Duration;
import java.util.Iterator;
import java.util.Map;
import java.util.Map.Entry;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.concurrent.Mutex;
import org.globalart.astrum.wonderland.AuthTicketCache;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.Maps;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import org.springframework.stereotype.Component;

/**
 * Manages cache (and expiration) of authentication tickets.
 *
 * @since 2.7
 */
@Component
public class LocalAuthTicketCache
    implements AuthTicketCache
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Mutex lock = new Mutex();

  private final Map<UserAuthToken, Long> tokens = Maps.newHashMap();

  private final Duration expireAfter;

  @Autowired
  public LocalAuthTicketCache(@Value(EXPIRE_VALUE) final Duration expireAfter) {
    this.expireAfter = checkNotNull(expireAfter);
    log.debug("Expire after: {}", expireAfter);
  }

  @VisibleForTesting
  public LocalAuthTicketCache() {
    this(Duration.ofSeconds(2));
  }

  private long now() {
    return System.currentTimeMillis();
  }

  /**
   * Expires any tokens older than {@link #expireAfter}.
   */
  private void expireTokens() {
    boolean trace = log.isTraceEnabled();
    long now = now();
    Iterator<Entry<UserAuthToken, Long>> iter = tokens.entrySet().iterator();
    while (iter.hasNext()) {
      Entry<UserAuthToken, Long> entry = iter.next();
      if (isTokenExpired(now, entry)) {
        iter.remove();
        if (trace) {
          log.trace("Expired token: {}", entry.getKey());
        }
      }
    }

    if (trace && !tokens.isEmpty()) {
      log.trace("Valid tokens:");
      for (Entry<UserAuthToken, Long> entry : tokens.entrySet()) {
        log.trace("  {}", entry.getKey());
      }
    }
  }

  @VisibleForTesting
  protected boolean isTokenExpired(final long now, final Entry<UserAuthToken, Long> entry) {
    long diff = now - entry.getValue();
    return diff > expireAfter.toMillis();
  }

  /**
   * Add token to the cache.
   */
  @Override
  public void add(final String user, final String token, final String realmName) {
    synchronized (lock) {
      expireTokens();
      UserAuthToken key = new UserAuthToken(user, token, realmName);
      // Sanity check we don't clobber tokens
      checkState(!tokens.containsKey(key), "Duplicate token"); // NON-NLS
      tokens.put(key, now());
    }
  }

  /**
   * Remove token from cache.
   *
   * @return {@code true} if the token existed (was added and not yet expired)
   */
  @Override
  public boolean remove(final String user, final String token, final String realmName) {
    synchronized (lock) {
      expireTokens();
      Long tmp = tokens.remove(new UserAuthToken(user, token, realmName));
      return tmp != null;
    }
  }
}
