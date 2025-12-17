
package org.globalart.astrum.security.authc;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

@Component
public class SecurityAuditor
    extends AuditorSupport
    implements EventAware
{
  public SecurityAuditor() {
    registerType(LoginEvent.class, "login");
    registerType(LogoutEvent.class, "logout");
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final SecurityEvent event) {
    AuditData data = new AuditData();
    data.setDomain(event.getRealm());
    data.setType(type(event.getClass()));
    data.getAttributes().put("principal", event.getPrincipal());
    record(data);
  }
}
