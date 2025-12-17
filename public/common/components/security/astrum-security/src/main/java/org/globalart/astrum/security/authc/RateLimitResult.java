
package org.globalart.astrum.security.authc;

/**
 * Result returned by {@link AuthRateLimiterService#checkAndRecord(String)} when a request is rate-limited.
 */
public record RateLimitResult(long retryAfterSeconds, int attemptCount)
{
}
