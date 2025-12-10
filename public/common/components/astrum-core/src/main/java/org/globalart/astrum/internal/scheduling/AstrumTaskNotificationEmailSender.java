
package org.globalart.astrum.internal.scheduling;

import java.util.List;
import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventAware.Asynchronous;
import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.scheduling.Task;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskNotificationCondition;
import org.globalart.astrum.scheduling.TaskNotificationMessageGenerator;
import org.globalart.astrum.scheduling.events.TaskEventStoppedDone;
import org.globalart.astrum.scheduling.events.TaskEventStoppedFailed;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.apache.commons.mail2.jakarta.Email;
import org.apache.commons.mail2.jakarta.SimpleEmail;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * {@link EventAware} that will send notification email (if necessary) in case of a completed or failed {@link Task}.
 */
@Component
public class AstrumTaskNotificationEmailSender
    implements EventAware, Asynchronous
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Provider<EmailManager> emailManager;

  private final Map<String, TaskNotificationMessageGenerator> taskNotificationMessageGenerators;

  @Autowired
  public AstrumTaskNotificationEmailSender(
      final Provider<EmailManager> emailManager,
      final List<TaskNotificationMessageGenerator> taskNotificationMessageGeneratorsList)
  {
    this.emailManager = checkNotNull(emailManager);
    this.taskNotificationMessageGenerators =
        QualifierUtil.buildQualifierBeanMap(checkNotNull(taskNotificationMessageGeneratorsList));
  }

  /**
   * Sends alert emails if necessary on failure.
   */
  @Subscribe
  @AllowConcurrentEvents
  public void on(final TaskEventStoppedFailed event) {
    final TaskInfo taskInfo = event.getTaskInfo();
    if (!haveAlertEmail(taskInfo)) {
      return;
    }
    String body = taskNotificationMessageGenerator(taskInfo.getTypeId()).failed(taskInfo, event.getFailureCause());
    log.trace("sending message {}", body);
    sendEmail("Task execution failure", taskInfo.getConfiguration().getAlertEmail(), body);
  }

  /**
   * Sends alert emails on completion.
   */
  @Subscribe
  @AllowConcurrentEvents
  public void on(final TaskEventStoppedDone event) {
    final TaskInfo taskInfo = event.getTaskInfo();
    if (!haveAlertEmail(taskInfo) ||
        taskInfo.getConfiguration().getNotificationCondition() == TaskNotificationCondition.FAILURE) {
      return;
    }
    String body = taskNotificationMessageGenerator(taskInfo.getTypeId()).completed(taskInfo);
    log.trace("sending message {}", body);
    sendEmail("Task execution completed", taskInfo.getConfiguration().getAlertEmail(), body);
  }

  private boolean haveAlertEmail(final TaskInfo taskInfo) {
    return taskInfo != null && taskInfo.getConfiguration().getAlertEmail() != null;
  }

  private void sendEmail(final String subject, final String address, final String body) {
    try {
      Email mail = new SimpleEmail();
      mail.setSubject(subject);
      mail.addTo(address);
      mail.setMsg(emailManager.get().constructMessage(body));
      emailManager.get().send(mail);
    }
    catch (Exception e) {
      log.warn("Failed to send email", e);
    }
  }

  private TaskNotificationMessageGenerator taskNotificationMessageGenerator(final String typeId) {
    TaskNotificationMessageGenerator result = taskNotificationMessageGenerators.get(typeId);
    if (result == null) {
      result = taskNotificationMessageGenerators.get(DefaultTaskNotificationMessageGenerator.ID);
    }
    return result;
  }
}
