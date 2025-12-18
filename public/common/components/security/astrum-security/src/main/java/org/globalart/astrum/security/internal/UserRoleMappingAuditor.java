
package org.globalart.astrum.security.internal;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.user.UserRoleMappingCreatedEvent;
import org.globalart.astrum.security.user.UserRoleMappingDeletedEvent;
import org.globalart.astrum.security.user.UserRoleMappingEvent;
import org.globalart.astrum.security.user.UserRoleMappingUpdatedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * User role-mapping auditor.
 *
 * @since 3.1
 */
@Component
public class UserRoleMappingAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.user-role-mapping";

  public UserRoleMappingAuditor() {
    registerType(UserRoleMappingCreatedEvent.class, CREATED_TYPE);
    registerType(UserRoleMappingDeletedEvent.class, DELETED_TYPE);
    registerType(UserRoleMappingUpdatedEvent.class, UPDATED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final UserRoleMappingEvent event) {
    if (isRecording()) {
      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(event.getUserId());

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("id", event.getUserId());
      attributes.put("source", event.getUserSource());
      attributes.put("roles", string(event.getRoles()));

      record(data);
    }
  }
}
