
package org.globalart.astrum.logging.task;

import org.junit.Test;
import org.slf4j.Logger;

import static org.hamcrest.CoreMatchers.instanceOf;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.mock;
import static org.globalart.astrum.logging.task.TaskLogType.BOTH;
import static org.globalart.astrum.logging.task.TaskLogType.ASTRUM_LOG_ONLY;
import static org.globalart.astrum.logging.task.TaskLogType.REPLICATION_LOGGING;
import static org.globalart.astrum.logging.task.TaskLogType.TASK_LOG_ONLY;
import static org.globalart.astrum.logging.task.TaskLogType.TASK_LOG_ONLY_WITH_PROGRESS;

public class TaskLoggerFactoryTest

{
  @Test
  public void testBoth() {
    TaskLogger taskLogger = TaskLoggerFactory.create(new Both(), mock(Logger.class), mock(TaskLogInfo.class));
    assertThat(taskLogger, instanceOf(SeparateTaskLogTaskLogger.class));
  }

  @Test
  public void testTaskLogOnly() {
    TaskLogger taskLogger = TaskLoggerFactory.create(new TaskLogOnly(), mock(Logger.class), mock(TaskLogInfo.class));
    assertThat(taskLogger, instanceOf(TaskLogOnlyTaskLogger.class));
  }

  @Test
  public void testReplicationLogging() {
    TaskLogger taskLogger =
        TaskLoggerFactory.create(new ReplicationLogging(), mock(Logger.class), mock(TaskLogInfo.class));
    assertThat(taskLogger, instanceOf(ReplicationTaskLogger.class));
  }

  @Test
  public void testTaskLogWithProgress() {
    TaskLogger taskLogger =
        TaskLoggerFactory.create(new TaskLogWithProgress(), mock(Logger.class), mock(TaskLogInfo.class));
    assertThat(taskLogger, instanceOf(TaskLogWithProgressLogger.class));
  }

  @Test
  public void testAstrumLogOnly() {
    TaskLogger taskLogger = TaskLoggerFactory.create(new AstrumLogOnly(), mock(Logger.class), mock(TaskLogInfo.class));
    assertThat(taskLogger, instanceOf(ProgressTaskLogger.class));
  }

  @Test
  public void testDefault() {
    TaskLogger taskLogger = TaskLoggerFactory.create(new Object(), mock(Logger.class), mock(TaskLogInfo.class));
    assertThat(taskLogger, instanceOf(SeparateTaskLogTaskLogger.class));
  }

  @TaskLogging(BOTH)
  private static final class Both
  {
  }

  @TaskLogging(TASK_LOG_ONLY)
  private static final class TaskLogOnly
  {
  }

  @TaskLogging(REPLICATION_LOGGING)
  private static final class ReplicationLogging
  {
  }

  @TaskLogging(TASK_LOG_ONLY_WITH_PROGRESS)
  private static final class TaskLogWithProgress
  {
  }

  @TaskLogging(ASTRUM_LOG_ONLY)
  private static final class AstrumLogOnly
  {
  }
}
