
package org.globalart.astrum.security.internal;

import java.util.HashSet;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.ClientInfo;
import org.globalart.astrum.security.authc.AuthenticationFailureReason;
import org.globalart.astrum.security.authc.AstrumAuthenticationEvent;

import com.google.common.collect.Sets;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Writes to the audit log for fired {@link AstrumAuthenticationEvent}
 *
 * @since 3.22
 */
@Component
public class AstrumAuthenticationEventAuditor
    extends AuditorSupport
    implements EventAware, EventAware.Asynchronous
{
  private static final String DOMAIN = "security.user";

  /* for now only log a subset of failure reasons */
  private static Set<AuthenticationFailureReason> AUDITABLE_FAILURE_REASONS = new HashSet<>();

  static {
    AUDITABLE_FAILURE_REASONS.add(AuthenticationFailureReason.INCORRECT_CREDENTIALS);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AstrumAuthenticationEvent event) {
    Set<AuthenticationFailureReason> failureReasonsToLog = getFailureReasonsToLog(event);

    if (isRecording() && !failureReasonsToLog.isEmpty()) {
      AuditData auditData = new AuditData();

      auditData.setType("authentication");
      auditData.setDomain(DOMAIN);
      auditData.setTimestamp(event.getEventDate());

      Map<String, Object> attributes = auditData.getAttributes();
      attributes.put("failureReasons", failureReasonsToLog);
      attributes.put("wasSuccessful", event.isSuccessful());

      if (event.getClientInfo() != null) {
        ClientInfo clientInfo = event.getClientInfo();
        attributes.put("userId", clientInfo.getUserid());
        attributes.put("remoteIp", clientInfo.getRemoteIP());
        attributes.put("userAgent", clientInfo.getUserAgent());
        attributes.put("path", clientInfo.getPath());
      }

      record(auditData);
    }
  }

  private Set<AuthenticationFailureReason> getFailureReasonsToLog(AstrumAuthenticationEvent event) {
    return Sets.intersection(event.getAuthenticationFailureReasons(), AUDITABLE_FAILURE_REASONS);
  }
}
