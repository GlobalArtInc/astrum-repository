
package org.globalart.astrum.scheduling.spi;

import java.util.Optional;

import org.globalart.astrum.scheduling.TaskInfo;

/**
 * Store for persisted Task state
 */
public interface TaskResultStateStore
{
  /**
   * Retrieve a state from the provided {@link TaskInfo}
   */
  Optional<TaskResultState> getState(TaskInfo taskInfo);

  void updateJobDataMap(TaskInfo taskInfo);

  default boolean isSupported() {
    return true;
  }
}
