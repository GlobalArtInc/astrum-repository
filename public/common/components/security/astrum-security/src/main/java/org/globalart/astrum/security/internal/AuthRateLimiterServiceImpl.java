
package org.globalart.astrum.security.internal;

import java.util.concurrent.TimeUnit;

import javax.annotation.Nullable;

import org.globalart.astrum.security.authc.AuthRateLimiterService;
import org.globalart.astrum.security.authc.RateLimitResult;

import com.google.common.cache.CacheBuilder;
import com.google.common.cache.CacheLoader;
import com.google.common.cache.LoadingCache;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import jakarta.inject.Singleton;

import static org.globalart.astrum.common.app.FeatureFlags.AUTH_RATE_LIMIT_ENABLED;

/**
 * Default implementation of {@link AuthRateLimiterService} using in-memory tracking with
 * exponential backoff. Tracks failed attempts per username; a request is blocked when
 * the username exceeds the configured failure threshold.
 *
 * <p>
 * Uses a Guava {@link Cache} with last-accessed expiration and soft values. Soft references
 * allow the GC to reclaim entries under memory pressure, preventing an attacker from exhausting
 * heap by submitting logins for arbitrary bogus usernames. Idle entries are automatically evicted
 * after {@code maxDelaySeconds} of inactivity.
 * </p>
 *
 * <p>
 * <strong>Cluster limitation:</strong> State is held in-memory on each node and is not
 * shared across cluster members. In a multi-node deployment an attacker can distribute
 * attempts across nodes and receive up to {@code maxAttempts} free tries per node before
 * any single node begins blocking. This is a known limitation of this implementation.
 * </p>
 */
@Component
@Singleton
@ConditionalOnProperty(name = AUTH_RATE_LIMIT_ENABLED, havingValue = "true", matchIfMissing = true)
public class AuthRateLimiterServiceImpl
    implements AuthRateLimiterService
{
  private static final Logger log = LoggerFactory.getLogger(AuthRateLimiterServiceImpl.class);

  private static final String USER_PREFIX = "user::";

  private static final String TOKEN_PREFIX = "token::";

  private final int maxAttempts;

  private final long baseDelaySeconds;

  private final long maxDelaySeconds;

  private final LoadingCache<String, AttemptRecord> attempts;

  public AuthRateLimiterServiceImpl(
      @Value("${astrum.auth.ratelimit.max-attempts:3}") final int maxAttempts,
      @Value("${astrum.auth.ratelimit.base-delay-seconds:30}") final long baseDelaySeconds,
      @Value("${astrum.auth.ratelimit.max-delay-seconds:900}") final long maxDelaySeconds,
      @Value("${astrum.auth.ratelimit.max-tracked-keys:10000}") final int maxTrackedKeys)
  {
    this.maxAttempts = maxAttempts;
    this.baseDelaySeconds = baseDelaySeconds;
    this.maxDelaySeconds = maxDelaySeconds;

    this.attempts = CacheBuilder.newBuilder()
        .expireAfterAccess(maxDelaySeconds, TimeUnit.SECONDS)
        .softValues()
        .maximumSize(maxTrackedKeys)
        .build(CacheLoader.from(AttemptRecord::new));
  }

  @Override
  @Nullable
  public RateLimitResult check(final String username) {
    return checkForKey(USER_PREFIX + username);
  }

  @Override
  @Nullable
  public RateLimitResult checkAndRecord(final String username) {
    return checkAndRecordForKey(USER_PREFIX + username);
  }

  @Override
  public void recordSuccess(final String username) {
    attempts.invalidate(USER_PREFIX + username);
    log.debug("Rate limit counter cleared on success for user '{}'", username);
  }

  @Override
  public void reset(final String username) {
    attempts.invalidate(USER_PREFIX + username);
    log.debug("Rate limit counter reset for user '{}'", username);
  }

  @Override
  @Nullable
  public RateLimitResult checkByToken(final String tokenHash) {
    return checkForKey(TOKEN_PREFIX + tokenHash);
  }

  @Override
  @Nullable
  public RateLimitResult checkAndRecordByToken(final String tokenHash) {
    return checkAndRecordForKey(TOKEN_PREFIX + tokenHash);
  }

  @Override
  public void recordSuccessByToken(final String tokenHash) {
    attempts.invalidate(TOKEN_PREFIX + tokenHash);
    log.debug("Rate limit counter cleared on success for token hash prefix '{}'",
        tokenHash.substring(0, Math.min(8, tokenHash.length())));
  }

  @Nullable
  private RateLimitResult checkForKey(final String key) {
    AttemptRecord record = attempts.getIfPresent(key);
    if (record == null) {
      return null;
    }
    synchronized (record) {
      if (record.failureCount <= maxAttempts) {
        return null;
      }
      long shift = record.failureCount - maxAttempts - 1;
      long retryAfter = (shift >= 63)
          ? maxDelaySeconds
          : Math.min(baseDelaySeconds * (1L << shift), maxDelaySeconds);
      return new RateLimitResult(retryAfter, record.failureCount);
    }
  }

  private RateLimitResult checkAndRecordForKey(final String key) {
    AttemptRecord record = attempts.getUnchecked(key);
    synchronized (record) {
      // Intentionally increment even when already rate-limited: each blocked attempt
      // advances the exponential backoff, discouraging repeated brute-force retries.
      record.failureCount++;

      if (record.failureCount <= maxAttempts) {
        return null;
      }

      long shift = record.failureCount - maxAttempts - 1;
      long retryAfter = (shift >= 63)
          ? maxDelaySeconds
          : Math.min(baseDelaySeconds * (1L << shift), maxDelaySeconds);

      // Log first block at INFO to leave a clear signal in production logs without spamming.
      if (record.failureCount == maxAttempts + 1) {
        log.info("Rate limiting key '{}': attempt={}, retryAfter={}s", key, record.failureCount, retryAfter);
      }
      else {
        log.debug("Rate limiting key '{}': attempt={}, retryAfter={}s", key, record.failureCount, retryAfter);
      }
      return new RateLimitResult(retryAfter, record.failureCount);
    }
  }

  private static final class AttemptRecord
  {
    int failureCount;
  }
}
