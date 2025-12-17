
package org.globalart.astrum.security.authc;

import javax.annotation.Nullable;

/**
 * Fired when an authentication attempt is blocked by the rate limiter.
 */
public class AuthRateLimitedEvent
{
  private final String username;

  private final int attemptCount;

  private final long retryAfterSeconds;

  @Nullable
  private final String sourceIp;

  private final String authMethod;

  public AuthRateLimitedEvent(
      final String username,
      final int attemptCount,
      final long retryAfterSeconds,
      @Nullable final String sourceIp,
      final String authMethod)
  {
    this.username = username;
    this.attemptCount = attemptCount;
    this.retryAfterSeconds = retryAfterSeconds;
    this.sourceIp = sourceIp;
    this.authMethod = authMethod;
  }

  public String getUsername() {
    return username;
  }

  public int getAttemptCount() {
    return attemptCount;
  }

  public long getRetryAfterSeconds() {
    return retryAfterSeconds;
  }

  @Nullable
  public String getSourceIp() {
    return sourceIp;
  }

  public String getAuthMethod() {
    return authMethod;
  }
}
