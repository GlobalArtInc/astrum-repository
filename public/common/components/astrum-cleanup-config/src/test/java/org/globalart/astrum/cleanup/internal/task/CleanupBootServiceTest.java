
package org.globalart.astrum.cleanup.internal.task;

import java.util.Date;
import java.util.List;

import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.schedule.Cron;
import org.globalart.astrum.scheduling.schedule.ScheduleFactory;

import com.google.common.collect.ImmutableList;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Collections.emptyList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.cleanup.internal.task.CleanupBootService.TASK_NAME;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CleanupBootServiceTest

{
  private TaskConfiguration taskConfig;

  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private ScheduleFactory scheduleFactory;

  @Mock
  private TaskInfo taskInfo;

  private CleanupBootService underTest;

  @Before
  public void setup() throws Exception {
    underTest = new CleanupBootService(taskScheduler);

    taskConfig = new TaskConfiguration();
    taskConfig.setTypeId(CleanupTaskDescriptor.TYPE_ID);
    taskConfig.setName(TASK_NAME);
    when(taskInfo.getConfiguration()).thenReturn(taskConfig);
    when(taskScheduler.listsTasks()).thenReturn(emptyList());

    when(taskScheduler.createTaskConfigurationInstance(CleanupTaskDescriptor.TYPE_ID)).thenReturn(taskConfig);
    when(taskScheduler.getScheduleFactory()).thenReturn(scheduleFactory);

    when(scheduleFactory.cron(any(), any())).thenAnswer(invokation -> {
      return new Cron((Date) invokation.getArguments()[0], (String) invokation.getArguments()[1]);
    });
  }

  @Test
  public void shouldCreateTaskOnStart() throws Exception {
    underTest.doStart();

    verify(taskScheduler).scheduleTask(any(), any());
  }

  @Test
  public void scheduleTaskDailyAt1am() throws Exception {
    underTest.doStart();

    verify(scheduleFactory, times(2)).cron(any(Date.class), eq("0 0 1 * * ?"));
  }

  @Test
  public void setTaskName() throws Exception {
    underTest.doStart();

    assertThat(taskConfig.getName(), is(equalTo(TASK_NAME)));
  }

  @Test
  public void doNotCreateTaskIfAlreadyExists() throws Exception {
    when(taskScheduler.listsTasks()).thenReturn(ImmutableList.of(taskInfo));

    underTest.doStart();

    verify(taskScheduler, never()).scheduleTask(any(), any());
  }

  @Test
  public void duplicatesRemoved() {
    TaskInfo nameMismatch = mockTask("foo", CleanupBootService.CRON);
    TaskInfo cronMismatch = mockTask(CleanupBootService.TASK_NAME, "1 0 1 * * ?");
    TaskInfo scheduleMismatch = mock(TaskInfo.class);
    when(scheduleMismatch.getConfiguration()).thenReturn(taskConfig);
    when(scheduleMismatch.getName()).thenReturn(CleanupBootService.TASK_NAME);

    TaskInfo matchA = mockTask(CleanupBootService.TASK_NAME, CleanupBootService.CRON);
    TaskInfo matchB = mockTask(CleanupBootService.TASK_NAME, CleanupBootService.CRON);

    List<TaskInfo> tasks = ImmutableList.of(nameMismatch, cronMismatch, scheduleMismatch, matchA, matchB);

    when(taskScheduler.listsTasks()).thenReturn(tasks);

    underTest.doStart();

    verify(nameMismatch, never()).remove();
    verify(cronMismatch, never()).remove();
    verify(scheduleMismatch, never()).remove();
    verify(matchA, never()).remove();
    verify(matchB).remove();

  }

  private static TaskInfo mockTask(final String name, final String cron) {
    TaskInfo task = mock(TaskInfo.class);
    when(task.getName()).thenReturn(name);
    TaskConfiguration taskConfig = new TaskConfiguration();
    taskConfig.setName(name);
    taskConfig.setTypeId(CleanupTaskDescriptor.TYPE_ID);
    when(task.getConfiguration()).thenReturn(taskConfig);
    when(task.getSchedule()).thenReturn(new Cron(new Date(), cron));
    return task;
  }
}
