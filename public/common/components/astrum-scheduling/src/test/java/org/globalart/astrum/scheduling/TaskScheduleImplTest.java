
package org.globalart.astrum.scheduling;

import java.util.List;

import jakarta.inject.Provider;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.scheduling.spi.SchedulerSPI;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Arrays.asList;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.not;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.scheduling.TaskSchedulerImpl.REPO_MOVE_TYPE_ID;

/**
 * Test class for {@link TaskSchedulerImpl}.
 *
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class TaskScheduleImplTest

{
  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private EventManager eventManager;

  @Mock
  private TaskFactory taskFactory;

  @Mock
  private Provider<SchedulerSPI> scheduler;

  @Mock
  private TaskInfo taskInfo1;

  @Mock
  private TaskInfo taskInfo2;

  @Mock
  private SchedulerSPI schedulerSPI;

  TaskSchedulerImpl underTest;

  @Before
  public void setup() {
    when(taskInfo1.getTypeId()).thenReturn(REPO_MOVE_TYPE_ID);
    when(taskInfo2.getTypeId()).thenReturn("anytype");

    when(scheduler.get()).thenReturn(schedulerSPI);
    when(schedulerSPI.listsTasks()).thenReturn(asList(taskInfo1, taskInfo2));
    when(schedulerSPI.getTaskById("1")).thenReturn(taskInfo1);
    when(schedulerSPI.getTaskById("2")).thenReturn(taskInfo2);

  }

  @Test
  public void listTasks_TaskDisabled() {
    underTest = new TaskSchedulerImpl(eventManager, taskFactory, scheduler, false);
    List<TaskInfo> taskList = underTest.listsTasks();
    assertThat(taskList.size(), is(1));
    assertThat(taskList.get(0).getTypeId(), not(REPO_MOVE_TYPE_ID));
  }

  @Test
  public void listTasks_TaskEnabled() {
    underTest = new TaskSchedulerImpl(eventManager, taskFactory, scheduler, true);
    List<TaskInfo> taskList = underTest.listsTasks();
    assertThat(taskList.size(), is(2));
  }

  @Test
  public void getTaskById_TaskDisabled() {
    underTest = new TaskSchedulerImpl(eventManager, taskFactory, scheduler, false);
    TaskInfo taskInfo = underTest.getTaskById("1");
    assertThat(taskInfo, is(nullValue()));

    taskInfo = underTest.getTaskById("2");
    assertThat(taskInfo, is(taskInfo2));
  }

  @Test
  public void getTaskById_TaskEnabled() {
    underTest = new TaskSchedulerImpl(eventManager, taskFactory, scheduler, true);

    TaskInfo taskInfo = underTest.getTaskById("1");
    assertThat(taskInfo, is(taskInfo1));
    assertThat(taskInfo1.getTypeId(), is(REPO_MOVE_TYPE_ID));

    taskInfo = underTest.getTaskById("2");
    assertThat(taskInfo, is(taskInfo2));
  }
}
