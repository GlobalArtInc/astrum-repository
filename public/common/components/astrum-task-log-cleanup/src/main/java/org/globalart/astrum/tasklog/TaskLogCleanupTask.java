
package org.globalart.astrum.tasklog;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.logging.task.TaskLogType.ASTRUM_LOG_ONLY;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Background task (hidden from users) that cleans up old log files.
 *
 * @since 3.5
 */
@Component
@TaskLogging(ASTRUM_LOG_ONLY)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TaskLogCleanupTask
    extends TaskSupport
    implements Cancelable
{
  private final TaskLogCleanup taskLogCleanup;

  @Autowired
  public TaskLogCleanupTask(final TaskLogCleanup taskLogCleanup) {
    this.taskLogCleanup = checkNotNull(taskLogCleanup);
  }

  @Override
  protected Void execute() throws Exception {
    taskLogCleanup.cleanup();
    return null;
  }

  @Override
  public String getMessage() {
    return "Remove old task log files";
  }
}
