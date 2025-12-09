
package org.globalart.astrum.internal.email;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.email.EmailConfigurationChangedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Email auditor.
 *
 * @since 3.1
 */
@Component
public class EmailAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "email";

  @Subscribe
  @AllowConcurrentEvents
  public void on(final EmailConfigurationChangedEvent event) {
    if (isRecording()) {
      EmailConfiguration configuration = event.getConfiguration();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(CHANGED_TYPE);
      data.setContext(SYSTEM_CONTEXT);

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("enabled", string(configuration.isEnabled()));
      attributes.put("host", configuration.getHost());
      attributes.put("port", string(configuration.getPort()));
      attributes.put("username", configuration.getUsername());
      attributes.put("fromAddress", configuration.getFromAddress());
      attributes.put("subjectPrefix", configuration.getSubjectPrefix());

      record(data);
    }
  }
}
