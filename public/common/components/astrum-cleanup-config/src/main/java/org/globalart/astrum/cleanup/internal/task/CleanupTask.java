
package org.globalart.astrum.cleanup.internal.task;

import javax.annotation.Nullable;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.cleanup.service.CleanupService;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.scheduling.CancelableByRecoveryMode;
import org.globalart.astrum.scheduling.RecoveryModeService;
import org.globalart.astrum.scheduling.TaskSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.logging.task.TaskLogType.TASK_LOG_ONLY;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Runs cleanup via the cleanup service.
 */
@Component
@TaskLogging(TASK_LOG_ONLY)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CleanupTask
    extends TaskSupport
    implements CancelableByRecoveryMode
{
  private final CleanupService cleanupService;

  private final RecoveryModeService recoveryModeService;

  @Autowired
  public CleanupTask(
      final CleanupService cleanupService,
      @Nullable final RecoveryModeService recoveryModeService)
  {
    this.cleanupService = checkNotNull(cleanupService);
    this.recoveryModeService = recoveryModeService;
  }

  @Override
  protected Object execute() {
    if (recoveryModeService != null) {
      recoveryModeService.ensureNotInRecoveryMode(getName());
    }

    log.info("Starting cleanup");

    cleanupService.cleanup(this::isCanceled);

    if (isCanceled()) {
      log.info("Cleanup was cancelled before it could finish");
    }
    else {
      log.info("Cleanup finished");
    }

    return null;
  }

  @Override
  public String getMessage() {
    return "Run repository cleanup";
  }
}
