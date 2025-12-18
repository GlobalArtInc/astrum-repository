
package org.globalart.astrum.security.internal;

import java.util.Map;
import java.util.Map.Entry;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.PrivilegeCreatedEvent;
import org.globalart.astrum.security.privilege.PrivilegeDeletedEvent;
import org.globalart.astrum.security.privilege.PrivilegeEvent;
import org.globalart.astrum.security.privilege.PrivilegeUpdatedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * {@link Privilege} auditor.
 *
 * @since 3.1
 */
@Component
public class PrivilegeAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.privilege";

  public PrivilegeAuditor() {
    registerType(PrivilegeCreatedEvent.class, CREATED_TYPE);
    registerType(PrivilegeDeletedEvent.class, DELETED_TYPE);
    registerType(PrivilegeUpdatedEvent.class, UPDATED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final PrivilegeEvent event) {
    if (isRecording()) {
      Privilege privilege = event.getPrivilege();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(privilege.getId());

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("id", privilege.getId());
      attributes.put("name", privilege.getName());
      attributes.put("type", privilege.getType());

      for (Entry<String, String> entry : privilege.getProperties().entrySet()) {
        attributes.put("property." + entry.getKey(), entry.getValue());
      }

      record(data);
    }
  }
}
