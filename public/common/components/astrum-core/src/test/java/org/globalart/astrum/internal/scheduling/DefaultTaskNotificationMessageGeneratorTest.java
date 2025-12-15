
package org.globalart.astrum.internal.scheduling;

import java.util.Date;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.common.template.TemplateParameters;
import org.globalart.astrum.scheduling.LastRunState;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskState;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests {@link DefaultTaskNotificationMessageGenerator}
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class DefaultTaskNotificationMessageGeneratorTest
{
  @Mock
  private TemplateHelper templateHelper;

  @Mock
  private TaskInfo taskInfo;

  @Mock
  private LastRunState lastRunState;

  @Mock
  private TaskConfiguration taskConfiguration;

  private DefaultTaskNotificationMessageGenerator underTest;

  @Before
  public void setUp() {
    underTest = new DefaultTaskNotificationMessageGenerator(templateHelper);

    // Setup common mock behavior
    when(taskInfo.getLastRunState()).thenReturn(lastRunState);
    when(taskInfo.getId()).thenReturn("test-task-id");
    when(taskInfo.getName()).thenReturn("Test Task");
    when(taskInfo.getConfiguration()).thenReturn(taskConfiguration);
    when(lastRunState.getEndState()).thenReturn(TaskState.OK);
    when(lastRunState.getRunStarted()).thenReturn(new Date());

    // Mock template helper to return the formatted parameters
    when(templateHelper.render(any(), any(TemplateParameters.class))).thenAnswer(invocation -> {
      TemplateParameters params = invocation.getArgument(1);
      // Return a simple string representation for testing
      return String.format("Task %s completed. Duration: %s",
          taskInfo.getName(),
          params.get().get("formattedDuration"));
    });
  }

  @Test
  public void completedFormatsDurationLessThanOneHour() {
    // Duration: 45 minutes 30 seconds = 2,730,000 ms
    long durationMillis = (45 * 60 + 30) * 1000L;
    when(lastRunState.getRunDuration()).thenReturn(durationMillis);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    verify(templateHelper).render(any(), any(TemplateParameters.class));
    assertThat(result, containsString("00:45:30"));
  }

  @Test
  public void completedFormatsDurationLessThan24Hours() {
    // Duration: 5 hours 23 minutes 45 seconds = 19,425,000 ms
    long durationMillis = (5 * 3600 + 23 * 60 + 45) * 1000L;
    when(lastRunState.getRunDuration()).thenReturn(durationMillis);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("05:23:45"));
  }

  @Test
  public void completedFormatsDurationExactly24Hours() {
    // Duration: exactly 24 hours = 86,400,000 ms
    long durationMillis = 24 * 60 * 60 * 1000L;
    when(lastRunState.getRunDuration()).thenReturn(durationMillis);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("24:00:00"));
  }

  @Test
  public void completedFormatsDurationMoreThan24Hours() {
    // Duration: 25 hours 15 minutes 30 seconds = 90,930,000 ms
    // This is the critical test case for ASTRUM-37907
    long durationMillis = (25 * 3600 + 15 * 60 + 30) * 1000L;
    when(lastRunState.getRunDuration()).thenReturn(durationMillis);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    // Should show 25:15:30, NOT 01:15:30 (which would be the buggy behavior)
    assertThat(result, containsString("25:15:30"));
  }

  @Test
  public void completedFormatsDurationMoreThan48Hours() {
    // Duration: 49 hours 5 minutes 10 seconds = 176,710,000 ms
    long durationMillis = (49 * 3600 + 5 * 60 + 10) * 1000L;
    when(lastRunState.getRunDuration()).thenReturn(durationMillis);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("49:05:10"));
  }

  @Test
  public void completedFormatsDurationMoreThan100Hours() {
    // Duration: 125 hours 59 minutes 59 seconds = 453,599,000 ms
    long durationMillis = (125 * 3600 + 59 * 60 + 59) * 1000L;
    when(lastRunState.getRunDuration()).thenReturn(durationMillis);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("125:59:59"));
  }

  @Test
  public void completedFormatsZeroDuration() {
    // Duration: 0 seconds
    when(lastRunState.getRunDuration()).thenReturn(0L);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("00:00:00"));
  }

  @Test
  public void completedFormatsOneSecondDuration() {
    // Duration: 1 second
    when(lastRunState.getRunDuration()).thenReturn(1000L);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("00:00:01"));
  }

  @Test
  public void completedFormatsSecondsOnly() {
    // Duration: 45 seconds
    when(lastRunState.getRunDuration()).thenReturn(45000L);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("00:00:45"));
  }

  @Test
  public void completedFormatsMinutesOnly() {
    // Duration: 5 minutes
    when(lastRunState.getRunDuration()).thenReturn(5 * 60 * 1000L);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("00:05:00"));
  }

  @Test
  public void completedFormatsHoursOnly() {
    // Duration: 2 hours
    when(lastRunState.getRunDuration()).thenReturn(2 * 60 * 60 * 1000L);

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("02:00:00"));
  }

  @Test
  public void completedPassesCorrectParametersToTemplate() {
    when(lastRunState.getRunDuration()).thenReturn(3600000L); // 1 hour

    String result = underTest.completed(taskInfo);

    assertThat(result, is(notNullValue()));
    assertThat(result, containsString("01:00:00"));
    verify(templateHelper).render(any(), any(TemplateParameters.class));
  }

  @Test
  public void failedPassesCorrectParametersToTemplate() {
    Throwable cause = new RuntimeException("Test exception");

    String result = underTest.failed(taskInfo, cause);

    assertThat(result, is(notNullValue()));
    verify(templateHelper).render(any(), any(TemplateParameters.class));
  }

  @Test
  public void failedHandlesNullCause() {
    String result = underTest.failed(taskInfo, null);

    assertThat(result, is(notNullValue()));
    verify(templateHelper).render(any(), any(TemplateParameters.class));
  }
}
