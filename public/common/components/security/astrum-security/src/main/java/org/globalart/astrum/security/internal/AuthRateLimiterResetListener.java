
package org.globalart.astrum.security.internal;

import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.authc.AuthRateLimiterService;
import org.globalart.astrum.security.authc.UserPasswordChanged;
import org.globalart.astrum.security.user.UserUpdatedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import jakarta.inject.Singleton;

import static org.globalart.astrum.common.app.FeatureFlags.AUTH_RATE_LIMIT_ENABLED;

/**
 * Clears the rate-limit counter for a user when their account is updated by an administrator,
 * allowing a locked-out user to regain access without a server restart.
 */
@Component
@Singleton
@ConditionalOnProperty(name = AUTH_RATE_LIMIT_ENABLED, havingValue = "true", matchIfMissing = true)
public class AuthRateLimiterResetListener
    implements EventAware
{
  private final AuthRateLimiterService rateLimiterService;

  public AuthRateLimiterResetListener(final AuthRateLimiterService rateLimiterService) {
    this.rateLimiterService = rateLimiterService;
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final UserUpdatedEvent event) {
    rateLimiterService.reset(event.getUser().getUserId());
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final UserPasswordChanged event) {
    rateLimiterService.reset(event.getUserId());
  }
}
