
package org.globalart.astrum.scheduling.internal;

import java.util.concurrent.Future;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.Freezable;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.spi.SchedulerSPI;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * Manages activation/passivation of the scheduler.
 *
 * @since 3.0
 */
@Component
@ManagedLifecycle(phase = TASKS)
@Order(Ordered.LOWEST_PRECEDENCE) // start scheduler at the end of this phase
public class TaskActivation
    extends StateGuardLifecycleSupport
    implements Freezable
{
  private final SchedulerSPI scheduler;

  private volatile boolean frozen;

  @Autowired
  public TaskActivation(final SchedulerSPI scheduler) {
    this.scheduler = checkNotNull(scheduler);
  }

  @Override
  protected void doStart() throws Exception {
    if (!isFrozen()) {
      scheduler.resume();
    }
  }

  @Override
  protected void doStop() throws Exception {
    scheduler.pause();
  }

  @Override
  public boolean isFrozen() {
    return frozen;
  }

  @Override
  public void freeze() {
    frozen = true;
    if (isStarted()) {
      scheduler.pause();
      scheduler.listsTasks()
          .stream()
          .filter(this::cancelOnFreeze)
          .filter(taskInfo -> !maybeCancel(taskInfo))
          .forEach(taskInfo -> log.warn("Unable to cancel task: {}", taskInfo.getName()));
    }
  }

  private boolean cancelOnFreeze(final TaskInfo taskInfo) {
    return taskInfo.getConfiguration() == null
        || !taskInfo.getConfiguration().getBoolean(TaskConfiguration.RUN_WHEN_FROZEN, false);
  }

  @Override
  public void unfreeze() {
    frozen = false;
    if (isStarted()) {
      scheduler.resume();
    }
  }

  private boolean maybeCancel(final TaskInfo taskInfo) {
    Future<?> future = taskInfo.getCurrentState().getFuture();
    return future == null || future.cancel(false);
  }
}
