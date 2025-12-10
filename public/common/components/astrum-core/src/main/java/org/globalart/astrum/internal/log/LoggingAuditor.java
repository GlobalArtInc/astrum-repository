
package org.globalart.astrum.internal.log;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.log.LoggerLevel;
import org.globalart.astrum.common.log.LoggerLevelChangedEvent;
import org.globalart.astrum.common.log.LoggersResetEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Logging auditor.
 *
 * @since 3.1
 */
@Component
public class LoggingAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "logging";

  @Subscribe
  @AllowConcurrentEvents
  public void on(final LoggersResetEvent event) {
    if (isRecording()) {
      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType("reset");
      data.setContext(SYSTEM_CONTEXT);
      record(data);
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final LoggerLevelChangedEvent event) {
    if (isRecording()) {
      String logger = event.getLogger();
      LoggerLevel level = event.getLevel();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(CHANGED_TYPE);
      data.setContext(logger);

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("logger", logger);
      attributes.put("level", string(level));
      record(data);
    }
  }
}
