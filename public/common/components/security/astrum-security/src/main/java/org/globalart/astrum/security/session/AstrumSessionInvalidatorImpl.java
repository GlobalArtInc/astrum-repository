
package org.globalart.astrum.security.session;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;

import org.apache.shiro.cache.Cache;
import org.apache.shiro.session.Session;
import org.apache.shiro.session.mgt.eis.CachingSessionDAO;
import org.apache.shiro.session.mgt.eis.SessionDAO;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.astrum.AstrumWebSessionManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.SESSION_ENABLED;

/**
 * Session-based invalidator for Shiro sessions.
 * Invalidates cached sessions when a user's password is changed.
 */
@Component
@ConditionalOnProperty(name = SESSION_ENABLED, havingValue = "true", matchIfMissing = true)
public class AstrumSessionInvalidatorImpl
    implements SessionInvalidator
{
  private static final Logger log = LoggerFactory.getLogger(AstrumSessionInvalidatorImpl.class);

  private static final String PRINCIPALS_SESSION_KEY =
      "org.apache.shiro.subject.support.DefaultSubjectContext_PRINCIPALS_SESSION_KEY";

  private final AstrumWebSessionManager sessionManager;

  private final AuditRecorder auditRecorder;

  public AstrumSessionInvalidatorImpl(
      final AstrumWebSessionManager sessionManager,
      final AuditRecorder auditRecorder)
  {
    this.sessionManager = checkNotNull(sessionManager);
    this.auditRecorder = checkNotNull(auditRecorder);
  }

  @Override
  public int invalidateSessionsForUser(final String username, final String userSource, final String reason) {
    log.info("Invalidating sessions for user '{}' (source '{}') due to {}", username, userSource, reason);

    try {
      SessionDAO sessionDAO = sessionManager.getSessionDAO();
      if (!(sessionDAO instanceof CachingSessionDAO)) {
        log.debug("SessionDAO is not a CachingSessionDAO, skipping session invalidation");
        return 0;
      }

      CachingSessionDAO cachingDAO = (CachingSessionDAO) sessionDAO;
      Cache<Serializable, Session> cache = cachingDAO.getActiveSessionsCache();

      if (cache == null) {
        log.debug("Active sessions cache is null, skipping session invalidation");
        return 0;
      }

      List<Session> toDelete = new ArrayList<>();
      for (Session session : cache.values()) {
        PrincipalCollection principals = getPrincipalsFromSession(session);
        if (principals != null && username.equals(principals.getPrimaryPrincipal())) {
          toDelete.add(session);
        }
      }
      toDelete.forEach(sessionDAO::delete);

      int count = toDelete.size();
      log.info("Invalidated {} session(s) for user '{}'", count, username);

      if (count > 0) {
        recordAuditEvent(username, count, reason);
      }

      return count;
    }
    catch (Exception e) {
      log.error("Failed to invalidate sessions for user '{}': {}", username, e.getMessage(), e);
      return 0;
    }
  }

  /**
   * Extract principals from a Shiro session.
   */
  private PrincipalCollection getPrincipalsFromSession(final Session session) {
    try {
      Object principals = session.getAttribute(PRINCIPALS_SESSION_KEY);
      if (principals instanceof PrincipalCollection) {
        return (PrincipalCollection) principals;
      }
    }
    catch (Exception e) {
      log.trace("Could not get principals from session: {}", e.getMessage());
    }
    return null;
  }

  private void recordAuditEvent(final String username, final int sessionCount, final String reason) {
    if (auditRecorder.isEnabled()) {
      AuditData data = new AuditData();
      data.setDomain("security.session");
      data.setType("user-session-invalidation");
      data.setContext(username);
      data.setTimestamp(new Date());
      data.setInitiator(username);

      data.getAttributes().put("username", username);
      data.getAttributes().put("sessionCount", String.valueOf(sessionCount));
      data.getAttributes().put("sessionType", "session");
      data.getAttributes().put("reason", reason);

      auditRecorder.record(data);
    }
  }
}
