
package org.globalart.astrum.cleanup.content;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.cleanup.storage.CleanupPolicy;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * {@link CleanupPolicy} auditor.
 *
 */
@Component
public class CleanupPolicyAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "cleanupPolicy";

  public CleanupPolicyAuditor() {
    registerType(CleanupPolicyCreatedEvent.class, CREATED_TYPE);
    registerType(CleanupPolicyUpdatedEvent.class, UPDATED_TYPE);
    registerType(CleanupPolicyDeletedEvent.class, DELETED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final CleanupPolicyEvent event) {
    if (isRecording()) {
      CleanupPolicy cleanupPolicy = event.getCleanupPolicy();

      AuditData data = getPolicyAuditData(event, cleanupPolicy);
      record(data);
    }
  }

  private AuditData getPolicyAuditData(final CleanupPolicyEvent event, final CleanupPolicy cleanupPolicy) {
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    data.setType(type(event.getClass()));
    data.setContext(cleanupPolicy.getName());

    Map<String, Object> attributes = data.getAttributes();
    attributes.put("format", cleanupPolicy.getFormat());
    attributes.put("description", cleanupPolicy.getNotes());
    attributes.put("criteria", cleanupPolicy.getCriteria());
    return data;
  }
}
