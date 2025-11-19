
package org.globalart.astrum.logging.task;

import javax.annotation.Nullable;

import java.util.Optional;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.slf4j.Logger;
import org.slf4j.MDC;
import org.slf4j.Marker;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.notNullValue;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.verify;
import static org.globalart.astrum.logging.task.ReplicationTaskLogger.REPLICATION_DISCRIMINATOR_ID;
import static org.globalart.astrum.logging.task.ReplicationTaskLogger.REPLICATION_LOG_LOCATION_PREFIX;
import static org.globalart.astrum.logging.task.TaskLogger.TASK_LOG_ONLY_MDC;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.ASTRUM_LOG_ONLY;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.TASK_LOG_ONLY;

public class ReplicationTaskLoggerTest
    extends ProgressTaskLoggerTest
{
  @Mock
  private Logger mockLogger;

  private ReplicationTaskLogger underTest;

  private TaskLogInfo taskLogInfo;

  private final String testPath = "test/log/replication";

  @Before
  public void setUp() throws Exception {
    super.setUp();
    this.taskLogInfo = createTaskLogInfoMock();
    underTest = new ReplicationTaskLogger(mockLogger, taskLogInfo);
    testPrintTaskDetails();
  }

  public void testPrintTaskDetails() {
    mockingTaskLogsHome(() -> {
      underTest.start();

      // details of run on replication log
      verifyLogInfoCalled(TASK_LOG_ONLY, "Replication run info:");
      verifyLogInfoCalled(TASK_LOG_ONLY, " Task ID: {}", taskLogInfo.getId());
      verifyLogInfoCalled(TASK_LOG_ONLY, " Type: {}", taskLogInfo.getTypeId());
      verifyLogInfoCalled(TASK_LOG_ONLY, " Name: {}", taskLogInfo.getName());
      verifyLogInfoCalled(TASK_LOG_ONLY, " Description: {}", taskLogInfo.getMessage());

      // verify info log was printed on astrum log
      verify(mockLogger).info(ASTRUM_LOG_ONLY, REPLICATION_LOG_LOCATION_PREFIX, "repositoryName",
          testPath + "/replication-repositoryName.log");

      // verify MDC has discriminator
      assertThat(MDC.get(REPLICATION_DISCRIMINATOR_ID), notNullValue());
      assertThat(MDC.get(REPLICATION_DISCRIMINATOR_ID), is("repositoryName"));
    });
  }

  @Test
  public void testDeleteMDCPropsOnFinish() {
    underTest.start();
    assertThat(MDC.get(TASK_LOG_ONLY_MDC), notNullValue());
    assertThat(MDC.get(REPLICATION_DISCRIMINATOR_ID), notNullValue());
    underTest.finish();
    assertThat(MDC.get(TASK_LOG_ONLY_MDC), nullValue());
    assertThat(MDC.get(REPLICATION_DISCRIMINATOR_ID), nullValue());
  }

  private void mockingTaskLogsHome(Runnable statement) {
    try (MockedStatic<TaskLogHome> mocked = mockStatic(TaskLogHome.class)) {
      mocked.when(TaskLogHome::getReplicationLogsHome).thenReturn(Optional.of(testPath));
      statement.run();
    }
  }

  private void verifyLogInfoCalled(Marker m, final String s) {
    verify(mockLogger).info(eq(m), eq(s));
  }

  private void verifyLogInfoCalled(final Marker m, final String s, final Object arg) {
    verify(mockLogger).info(eq(m), eq(s), eq(arg));
  }

  @After
  public void tearDown() throws Exception {
    underTest.finish();
  }

  private TaskLogInfo createTaskLogInfoMock() {
    return new TaskLogInfo()
    {
      @Override
      public String getId() {
        return "taskid";
      }

      @Override
      public String getTypeId() {
        return "typeId";
      }

      @Override
      public String getName() {
        return "name";
      }

      @Override
      public String getMessage() {
        return "message";
      }

      @Override
      public String toString() {
        return "toString";
      }

      @Nullable
      @Override
      public String getString(final String key) {
        return "repositoryName";
      }

      @Override
      public boolean getBoolean(final String key, final boolean defaultValue) {
        return false;
      }

      @Override
      public int getInteger(final String key, final int defaultValue) {
        return 0;
      }
    };
  }
}
