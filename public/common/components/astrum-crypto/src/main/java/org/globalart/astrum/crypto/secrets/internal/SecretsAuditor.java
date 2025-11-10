
package org.globalart.astrum.crypto.secrets.internal;

import java.util.Map;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.crypto.secrets.ActiveKeyChangeEvent;

import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

@Component
public class SecretsAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.secrets";

  @Subscribe
  public void on(final ActiveKeyChangeEvent event) {
    if (isRecording() && !EventHelper.isReplicating()) {
      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(CHANGED_TYPE);
      data.setContext(SYSTEM_CONTEXT);

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("newKeyId", event.getNewKeyId());
      attributes.put("previousKeyId", event.getPreviousKeyId());
      attributes.put("userId", event.getUserId());
      record(data);
    }
  }
}
