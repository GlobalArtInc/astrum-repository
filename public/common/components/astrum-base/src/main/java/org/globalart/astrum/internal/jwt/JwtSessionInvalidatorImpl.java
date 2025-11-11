
package org.globalart.astrum.internal.jwt;

import java.time.OffsetDateTime;
import java.util.Date;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.security.JwtHelper;
import org.globalart.astrum.security.jwt.JwtSessionRevocationService;
import org.globalart.astrum.security.session.SessionInvalidator;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;

/**
 * JWT-mode session invalidator.
 *
 * <p>
 * Records a user-wide invalidation cutoff on password change. Because JWTs are stateless,
 * Astrum does not track active JWTs; instead {@code JwtSecurityFilter} consults the cutoff
 * recorded here against each presented JWT's {@code iat} claim and rejects pre-cutoff tokens.
 * This invalidates every JWT ever issued to the user before the password change, on every
 * device, independent of which node processed the change.
 */
@Component
@ConditionalOnProperty(name = JWT_ENABLED, havingValue = "true")
public class JwtSessionInvalidatorImpl
    implements SessionInvalidator
{
  private static final Logger log = LoggerFactory.getLogger(JwtSessionInvalidatorImpl.class);

  private final JwtSessionRevocationService revocationService;

  private final JwtHelper jwtHelper;

  private final AuditRecorder auditRecorder;

  public JwtSessionInvalidatorImpl(
      final JwtSessionRevocationService revocationService,
      final JwtHelper jwtHelper,
      final AuditRecorder auditRecorder)
  {
    this.revocationService = checkNotNull(revocationService);
    this.jwtHelper = checkNotNull(jwtHelper);
    this.auditRecorder = checkNotNull(auditRecorder);
  }

  @Override
  public int invalidateSessionsForUser(final String username, final String userSource, final String reason) {
    log.info("Invalidating JWT sessions for user '{}' (source '{}') due to {}",
        username, userSource, reason);

    try {
      OffsetDateTime cutoff = OffsetDateTime.now();
      OffsetDateTime validUntil = cutoff.plusSeconds(jwtHelper.getExpirySeconds());

      revocationService.invalidateUser(username, userSource, cutoff, validUntil);

      recordAuditEvent(username, reason);
      return 1;
    }
    catch (Exception e) {
      log.error("Failed to record JWT user invalidation for user '{}' (source '{}'): {}",
          username, userSource, e.getMessage(), e);
      return 0;
    }
  }

  private void recordAuditEvent(final String username, final String reason) {
    if (auditRecorder.isEnabled()) {
      AuditData data = new AuditData();
      data.setDomain("security.session");
      data.setType("user-session-invalidation");
      data.setContext(username);
      data.setTimestamp(new Date());
      data.setInitiator(username);

      data.getAttributes().put("username", username);
      data.getAttributes().put("sessionCount", "1");
      data.getAttributes().put("sessionType", "jwt");
      data.getAttributes().put("reason", reason);

      auditRecorder.record(data);
    }
  }
}
