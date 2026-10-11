package org.globalart.astrum.accesstoken.internal;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Records access tokens being created and revoked.
 */
@Component
public class AccessTokenAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.accesstoken";

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AccessTokenEvent event) {
    if (!isRecording()) {
      return;
    }
    AccessTokenData token = event.getToken();
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    data.setType(event.getType() == AccessTokenEvent.Type.CREATED ? CREATED_TYPE : DELETED_TYPE);
    data.setContext(token.getUsername());
    Map<String, Object> attributes = data.getAttributes();
    attributes.put("id", token.getId());
    attributes.put("name", token.getName());
    attributes.put("owner", token.getUsername());
    attributes.put("scope", token.getScope());
    attributes.put("expires", string(token.getExpires()));
    record(data);
  }
}
