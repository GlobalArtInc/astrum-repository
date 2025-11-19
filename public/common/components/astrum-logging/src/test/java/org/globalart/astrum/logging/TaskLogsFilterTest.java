
package org.globalart.astrum.logging;

import org.globalart.astrum.logging.task.TaskLogger;
import org.globalart.astrum.logging.task.TaskLoggerHelper;
import org.globalart.astrum.logging.task.TaskLoggingEvent;

import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.classic.spi.LoggingEvent;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.slf4j.MDC;
import org.slf4j.Marker;

import static ch.qos.logback.core.spi.FilterReply.DENY;
import static ch.qos.logback.core.spi.FilterReply.NEUTRAL;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.junit.Assert.assertNotNull;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.verify;
import static org.globalart.astrum.logging.task.TaskLogger.LOGBACK_TASK_DISCRIMINATOR_ID;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.INTERNAL_PROGRESS;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.ASTRUM_LOG_ONLY;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.PROGRESS;

@RunWith(MockitoJUnitRunner.Silent.class)
public class TaskLogsFilterTest

{
  private static final String TEST_MESSAGE = "test message";

  private static final Object[] TEST_ARGS = new Object[]{"x"};

  @Mock
  private TaskLogger taskLogger;

  private TaskLogsFilter taskLogsFilter;

  @Before
  public void setUp() {
    taskLogsFilter = new TaskLogsFilter();
  }

  @After
  public void tearDown() {
    MDC.remove(LOGBACK_TASK_DISCRIMINATOR_ID);
    if (TaskLoggerHelper.get() != null) {
      TaskLoggerHelper.finish();
    }
  }

  @Test
  public void testNotATask() {
    // not a task
    MDC.remove(LOGBACK_TASK_DISCRIMINATOR_ID);
    assertThat(taskLogsFilter.decide(eventWithMarkerOf(null)), equalTo(DENY));
  }

  @Test
  public void testIsAAstrumLog() {
    startTask();
    assertThat(taskLogsFilter.decide(eventWithMarkerOf(ASTRUM_LOG_ONLY)), equalTo(DENY));
  }

  @Test
  public void testIsInternalProgress() {
    startTask();
    assertThat(taskLogsFilter.decide(eventWithMarkerOf(INTERNAL_PROGRESS)), equalTo(DENY));
  }

  @Test
  public void testIsProgress() {
    startTask();
    assertThat(taskLogsFilter.decide(eventWithMarkerOf(PROGRESS)), equalTo(NEUTRAL));
    assertNotNull(TaskLoggerHelper.get());

    ArgumentCaptor<TaskLoggingEvent> argumentCaptor = ArgumentCaptor.forClass(TaskLoggingEvent.class);
    verify(taskLogger).progress(argumentCaptor.capture());
    TaskLoggingEvent taskLoggingEvent = argumentCaptor.getValue();
    assertNotNull(taskLoggingEvent);
    assertThat(taskLoggingEvent.getMessage(), equalTo(TEST_MESSAGE));
    assertThat(taskLoggingEvent.getArgumentArray(), equalTo(TEST_ARGS));
  }

  @Test
  public void testNotProgress() {
    startTask();
    assertThat(taskLogsFilter.decide(eventWithMarkerOf(null)), equalTo(NEUTRAL));
    assertNotNull(TaskLoggerHelper.get());
  }

  private void startTask() {
    // default is a task
    MDC.put(LOGBACK_TASK_DISCRIMINATOR_ID, "taskId");

    TaskLoggerHelper.start(taskLogger);
  }

  private ILoggingEvent eventWithMarkerOf(final Marker marker) {
    LoggingEvent event = new LoggingEvent();
    event.setLoggerName("TaskLogsFilterTest");
    event.setMessage(TEST_MESSAGE);
    event.addMarker(marker);
    event.setArgumentArray(TEST_ARGS);
    return event;
  }
}
