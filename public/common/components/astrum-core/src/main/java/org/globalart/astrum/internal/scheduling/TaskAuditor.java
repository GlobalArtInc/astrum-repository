
package org.globalart.astrum.internal.scheduling;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.events.TaskBlockedEvent;
import org.globalart.astrum.scheduling.events.TaskDeletedEvent;
import org.globalart.astrum.scheduling.events.TaskEvent;
import org.globalart.astrum.scheduling.events.TaskEventCanceled;
import org.globalart.astrum.scheduling.events.TaskEventStarted;
import org.globalart.astrum.scheduling.events.TaskEventStoppedCanceled;
import org.globalart.astrum.scheduling.events.TaskEventStoppedDone;
import org.globalart.astrum.scheduling.events.TaskEventStoppedFailed;
import org.globalart.astrum.scheduling.events.TaskScheduledEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * Task auditor.
 *
 * @since 3.1
 */
@Component
public class TaskAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "tasks";

  public TaskAuditor() {
    // NOTE: scheduled is fired in case of task created or task updated; there is no easy way to determine which atm
    registerType(TaskScheduledEvent.class, "scheduled");
    registerType(TaskEventStarted.class, "started");
    registerType(TaskEventStoppedDone.class, "finished");
    registerType(TaskEventStoppedFailed.class, "failed");
    registerType(TaskEventCanceled.class, "cancel-requested");
    registerType(TaskEventStoppedCanceled.class, "canceled");
    registerType(TaskDeletedEvent.class, DELETED_TYPE);
    registerType(TaskBlockedEvent.class, "blocked");
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final TaskEvent event) {
    if (isRecording()) {
      TaskInfo task = event.getTaskInfo();
      TaskConfiguration configuration = task.getConfiguration();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(configuration.getTypeName());

      Map<String, Object> attributes = data.getAttributes();
      // TaskInfo.{id/name/message} are all delegates to configuration
      attributes.put("schedule", string(task.getSchedule()));
      attributes.put("currentState", string(task.getCurrentState()));
      attributes.put("lastRunState", string(task.getLastRunState()));

      // TODO: may want to use TaskDescriptor to provider better comprehension of the configuration
      // TODO: ... for now though, just include everything its simpler
      attributes.putAll(configuration.asMap());

      record(data);
    }
  }
}
