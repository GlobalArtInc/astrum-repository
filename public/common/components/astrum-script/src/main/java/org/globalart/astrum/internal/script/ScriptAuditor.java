
package org.globalart.astrum.internal.script;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.script.Script;
import org.globalart.astrum.script.ScriptCreatedEvent;
import org.globalart.astrum.script.ScriptDeletedEvent;
import org.globalart.astrum.script.ScriptEvent;
import org.globalart.astrum.script.ScriptUpdatedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * {@link Script} auditor.
 *
 * @since 3.1
 */
@Component
public class ScriptAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "script";

  public ScriptAuditor() {
    registerType(ScriptCreatedEvent.class, AuditorSupport.CREATED_TYPE);
    registerType(ScriptDeletedEvent.class, AuditorSupport.DELETED_TYPE);
    registerType(ScriptUpdatedEvent.class, AuditorSupport.UPDATED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ScriptEvent event) {
    if (isRecording()) {
      Script script = event.getScript();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(script.getName());

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("name", script.getName());
      attributes.put("type", script.getType());

      record(data);
    }
  }
}
