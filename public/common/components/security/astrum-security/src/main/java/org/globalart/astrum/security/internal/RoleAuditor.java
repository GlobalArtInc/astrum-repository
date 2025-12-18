
package org.globalart.astrum.security.internal;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.role.Role;
import org.globalart.astrum.security.role.RoleCreatedEvent;
import org.globalart.astrum.security.role.RoleDeletedEvent;
import org.globalart.astrum.security.role.RoleEvent;
import org.globalart.astrum.security.role.RoleUpdatedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * {@link Role} auditor.
 *
 * @since 3.1
 */
@Component
public class RoleAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.role";

  public RoleAuditor() {
    registerType(RoleCreatedEvent.class, CREATED_TYPE);
    registerType(RoleDeletedEvent.class, DELETED_TYPE);
    registerType(RoleUpdatedEvent.class, UPDATED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final RoleEvent event) {
    if (isRecording()) {
      Role role = event.getRole();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(role.getRoleId());

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("id", role.getRoleId());
      attributes.put("name", role.getName());
      attributes.put("source", role.getSource());
      attributes.put("roles", string(role.getRoles()));
      attributes.put("privileges", string(role.getPrivileges()));

      record(data);
    }
  }
}
