
package org.globalart.astrum.security.internal;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.authc.AuthRateLimitedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import jakarta.inject.Singleton;

import static org.globalart.astrum.common.app.FeatureFlags.AUTH_RATE_LIMIT_ENABLED;

/**
 * Audits {@link AuthRateLimitedEvent}s to the Astrum audit log.
 */
@Component
@Singleton
@ConditionalOnProperty(name = AUTH_RATE_LIMIT_ENABLED, havingValue = "true", matchIfMissing = true)
public class AuthRateLimitAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.auth.ratelimit";

  public AuthRateLimitAuditor() {
    registerType(AuthRateLimitedEvent.class, "blocked");
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AuthRateLimitedEvent event) {
    if (isRecording()) {
      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(event.getUsername());
      Map<String, Object> attributes = data.getAttributes();
      attributes.put("username", event.getUsername());
      attributes.put("attemptCount", event.getAttemptCount());
      attributes.put("retryAfterSeconds", event.getRetryAfterSeconds());
      if (event.getSourceIp() != null) {
        attributes.put("sourceIp", event.getSourceIp());
      }
      attributes.put("authMethod", event.getAuthMethod());
      record(data);
    }
  }
}
