
package org.globalart.astrum.quartz.internal.upgrades;

import java.util.UUID;

import org.globalart.astrum.quartz.SleeperTask;
import org.globalart.astrum.quartz.SleeperTaskDescriptor;
import org.globalart.astrum.quartz.TaskSchedulerTestSupport;
import org.globalart.astrum.quartz.internal.task.QuartzTaskInfo;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;

import org.mockito.MockedStatic;
import org.quartz.JobKey;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.CALLS_REAL_METHODS;
import static org.mockito.Mockito.mockStatic;

/**
 * Base test support class for {@link QuartzJobKeyUnificationTask} tests.
 * Provides shared helper methods for creating legacy tasks and running migrations.
 */
public abstract class QuartzJobKeyUnificationTaskTestSupport
    extends TaskSchedulerTestSupport
{
  protected static final long WAIT_TIMEOUT = 20000L; // 20 seconds

  /**
   * Creates a "legacy" task with mismatched job_name and task_configuration_id by temporarily mocking JobKey.jobKey()
   * to return a different UUID.
   */
  protected QuartzTaskInfo createLegacyTask(String taskName) {
    try (MockedStatic<JobKey> mockedJobKey = mockStatic(JobKey.class, CALLS_REAL_METHODS)) {
      mockedJobKey.when(() -> JobKey.jobKey(anyString(), anyString()))
          .thenReturn(new JobKey(UUID.randomUUID().toString(), "astrum"));

      return createNamedTask(taskName);
    }
  }

  /**
   * Creates a named SleeperTask with manual schedule (stays in WAITING state).
   */
  protected QuartzTaskInfo createNamedTask(String taskName) {
    TaskConfiguration config = taskScheduler.createTaskConfigurationInstance(SleeperTaskDescriptor.TYPE_ID);
    config.setName(taskName);
    config.setString(SleeperTask.RESULT_KEY, RESULT);
    return (QuartzTaskInfo) taskScheduler.scheduleTask(config, taskScheduler.getScheduleFactory().manual());
  }

  /**
   * Runs the QuartzJobKeyUnificationTask and returns its TaskInfo.
   */
  protected TaskInfo runMigrationTask() {
    TaskConfiguration config =
        taskScheduler.createTaskConfigurationInstance(QuartzJobKeyUnificationTaskDescriptor.TYPE_ID);
    config.setName("Job Key Migration Test");
    return taskScheduler.scheduleTask(config, taskScheduler.getScheduleFactory().now());
  }

  /**
   * Finds a task by its name.
   */
  protected QuartzTaskInfo findTaskByName(String name) {
    return taskScheduler.listsTasks()
        .stream()
        .filter(t -> name.equals(t.getName()))
        .findFirst()
        .map(QuartzTaskInfo.class::cast)
        .orElseThrow(() -> new AssertionError("Task not found: " + name));
  }

  /**
   * Asserts that a task is unified (job_name == config_id).
   */
  protected void assertTaskUnified(QuartzTaskInfo task) {
    assertThat(task.getName() + " should be unified",
        task.getJobKey().getName(), is(task.getConfiguration().getId()));
  }

  /**
   * Asserts that a task is NOT unified (legacy - job_name != config_id).
   */
  protected void assertTaskLegacy(QuartzTaskInfo task) {
    assertThat(task.getName() + " should be legacy (not unified)",
        task.getJobKey().getName(), is(not(task.getConfiguration().getId())));
  }
}
