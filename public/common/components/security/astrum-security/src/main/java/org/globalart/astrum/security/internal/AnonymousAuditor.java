
package org.globalart.astrum.security.internal;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousConfigurationChangedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Anonymous auditor.
 *
 * @since 3.1
 */
@Component
public class AnonymousAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.anonymous";

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AnonymousConfigurationChangedEvent event) {
    if (isRecording()) {
      AnonymousConfiguration configuration = event.getConfiguration();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(CHANGED_TYPE);
      data.setContext(SYSTEM_CONTEXT);

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("enabled", string(configuration.isEnabled()));
      attributes.put("userId", configuration.getUserId());
      attributes.put("realm", configuration.getRealmName());

      record(data);
    }
  }
}
