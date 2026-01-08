
package org.globalart.astrum.scheduling;

import java.util.List;

import javax.annotation.Nullable;

/**
 * Factory for creating {@link Task} instances.
 *
 * @see TaskDescriptor
 * @see TaskConfiguration
 */
public interface TaskFactory
{
  /**
   * Returns an immutable list of all detected descriptors.
   */
  List<TaskDescriptor> getDescriptors();

  /**
   * Find a descriptor by its type-id; null if not found.
   */
  @Nullable
  TaskDescriptor findDescriptor(String typeId);

  /**
   * Create a task from given configuration.
   */
  Task create(TaskConfiguration config, TaskInfo taskInfo);
}
