
package org.globalart.astrum.scheduling;

/**
 * A helper for upgrades which require scheduling a task after Astrum starts
 */
public interface UpgradeTaskScheduler
{
  /**
   * Schedule a task to run after Astrum starts. The task will block the upgrade task queue until it completes,
   * preventing subsequent upgrade tasks from running concurrently.
   */
  void schedule(TaskConfiguration configuration);

  /**
   * Schedule a task to run after Astrum starts.
   *
   * <p>
   * Note: the default implementation delegates to {@link #schedule(TaskConfiguration)} and ignores the
   * {@code blockQueue} flag. Implementations that wish to honour the flag must override this method.
   * Calling this default with {@code blockQueue=false} will silently fall back to blocking behaviour.
   * </p>
   *
   * @param configuration the task configuration
   * @param blockQueue if {@code true}, the task blocks the upgrade task queue until it completes;
   *          if {@code false}, subsequent tasks in the queue are not held back by this task
   */
  default void schedule(TaskConfiguration configuration, boolean blockQueue) {
    schedule(configuration);
  }

  /**
   * Create a configuration for the given type-id.
   */
  TaskConfiguration createTaskConfigurationInstance(String typeId);
}
