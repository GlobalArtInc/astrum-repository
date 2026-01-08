
package org.globalart.astrum.scheduling.internal.upgrade.datastore;

import java.util.Map;

import static com.google.common.base.Preconditions.checkNotNull;

public class UpgradeTaskData
{
  /**
   * Configuration key used to mark a task as non-blocking. When set to {@code "false"} in the task configuration map,
   * the upgrade task queue will not wait for this task to complete before starting the next queued task.
   */
  public static final String BLOCK_QUEUE_KEY = "upgrade.task.blockQueue";

  private Integer id;

  private String taskId;

  private String status;

  private Map<String, String> configuration;

  public UpgradeTaskData() {
    // deserialization
  }

  public UpgradeTaskData(final String taskId, final Map<String, String> configuration) {
    this.taskId = checkNotNull(taskId);
    this.configuration = checkNotNull(configuration);
  }

  /**
   * The configuration of the task to be run
   */
  public Map<String, String> getConfiguration() {
    return configuration;
  }

  public Integer getId() {
    return id;
  }

  /**
   * A string indicating the last task status
   */
  public String getStatus() {
    return status;
  }

  /**
   * Matches the ID from TaskInfo
   */
  public String getTaskId() {
    return taskId;
  }

  public void setConfiguration(final Map<String, String> configuration) {
    this.configuration = configuration;
  }

  public void setId(final Integer id) {
    this.id = id;
  }

  public void setStatus(final String status) {
    this.status = status;
  }

  public void setTaskId(final String taskId) {
    this.taskId = taskId;
  }

}
