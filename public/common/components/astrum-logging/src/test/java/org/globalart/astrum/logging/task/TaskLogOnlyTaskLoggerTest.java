
package org.globalart.astrum.logging.task;

import org.junit.Test;
import org.mockito.InjectMocks;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.slf4j.Logger;
import org.slf4j.MDC;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.startsWith;
import static org.mockito.Mockito.verify;
import static org.globalart.astrum.logging.task.SeparateTaskLogTaskLogger.TASK_LOG_LOCATION_PREFIX;
import static org.globalart.astrum.logging.task.TaskLogger.TASK_LOG_ONLY_MDC;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.ASTRUM_LOG_ONLY;

@RunWith(MockitoJUnitRunner.Silent.class)
public class TaskLogOnlyTaskLoggerTest

{
  @Mock
  private Logger log;

  @Mock
  private TaskLogInfo taskLogInfo;

  @InjectMocks
  private TaskLogOnlyTaskLogger taskLogOnlyTaskLogger;

  @Test
  public void mdcShouldContainTaskLogOnlyKey() {

    assertThat(MDC.get(TASK_LOG_ONLY_MDC), equalTo("true"));
  }

  @Test
  public void shouldWriteLogLocationToAstrumLog() {
    taskLogOnlyTaskLogger.writeLogFileNameToAstrumLog();

    verify(log).info(eq(ASTRUM_LOG_ONLY), startsWith(TASK_LOG_LOCATION_PREFIX));
    assertThat(MDC.get(TASK_LOG_ONLY_MDC), equalTo("true"));
  }
}
