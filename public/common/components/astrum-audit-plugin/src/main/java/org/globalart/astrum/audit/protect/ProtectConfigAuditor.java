
package org.globalart.astrum.audit.protect;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

@Component
public class ProtectConfigAuditor
    extends AuditorSupport
    implements EventAware
{
  @Subscribe
  @AllowConcurrentEvents
  public void on(final ProtectConfigChangedEvent event) {
    if (isRecording()) {
      AuditData data = new AuditData();
      data.setDomain(event.getDomain());
      data.setType(event.getType());
      data.setContext(event.getContext());
      data.setAttributes(Map.of(
          "from", event.getFromValue(),
          "to", event.getToValue()));
      record(data);
    }
  }
}
