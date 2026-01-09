
package org.globalart.astrum.scheduling;

import java.util.Optional;

import org.globalart.astrum.scheduling.spi.TaskResultState;
import org.globalart.astrum.scheduling.spi.TaskResultStateStore;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Arrays.asList;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.scheduling.TaskState.RUNNING;

@RunWith(MockitoJUnitRunner.Silent.class)
public class TaskUtilsTest

{
  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private TaskInfo taskInfo;

  @Mock
  private TaskConfiguration taskConfiguration;

  @Mock
  private TaskResultStateStore taskResultStateStore;

  @Mock
  private CurrentState currentState;

  private TaskUtils underTest;

  @Before
  public void setup() {
    when(taskScheduler.listsTasks()).thenReturn(asList(taskInfo));
    when(taskInfo.getId()).thenReturn("taskId");
    when(taskInfo.getName()).thenReturn("taskName");
    when(taskInfo.getTypeId()).thenReturn("taskTypeId");
    when(taskInfo.getConfiguration()).thenReturn(taskConfiguration);
    when(taskInfo.getCurrentState()).thenReturn(currentState);
    when(currentState.getState()).thenReturn(RUNNING);
    when(taskConfiguration.getString("key")).thenReturn("value");
    when(taskResultStateStore.isSupported()).thenReturn(false);

    underTest = new TaskUtils(() -> taskScheduler, taskResultStateStore);
  }

  @Test
  public void testCheckForConflictingTasks_ignoresMatchingWithSameId() {
    underTest
        .checkForConflictingTasks("taskId", "taskName", asList("taskTypeId"), ImmutableMap.of("key", asList("value")));
  }

  @Test
  public void testCheckForConflictingTasks_matchingTaskDiffConfigKey() {
    underTest.checkForConflictingTasks("taskId2", "taskName2", asList("taskTypeId"),
        ImmutableMap.of("key2", asList("value")));
  }

  @Test
  public void testCheckForConflictingTasks_matchingTaskDiffConfigValue() {
    underTest.checkForConflictingTasks("taskId2", "taskName2", asList("taskTypeId"),
        ImmutableMap.of("key", asList("value2")));
  }

  @Test
  public void testCheckForConflictingTasks_matchingTaskNotRunning() {
    when(currentState.getState()).thenReturn(TaskState.WAITING);
    underTest.checkForConflictingTasks("taskId2", "taskName2", asList("taskTypeId"),
        ImmutableMap.of("key", asList("value")));
  }

  @Test(expected = IllegalStateException.class)
  public void testCheckForConflictingTasks_conflictingTask() {
    underTest.checkForConflictingTasks("taskId2", "taskName2", asList("taskTypeId"),
        ImmutableMap.of("key", asList("value")));
  }

  @Test(expected = IllegalStateException.class)
  public void testCheckForConflictingTasks_conflictingTaskMultipleTypeIds() {
    underTest.checkForConflictingTasks("taskId2", "taskName2", asList("preTaskTypeId", "taskTypeId"),
        ImmutableMap.of("key", asList("value")));
  }

  @Test(expected = IllegalStateException.class)
  public void testCheckForConflictingTasks_conflictingTaskMultipleConfigKeys() {
    underTest.checkForConflictingTasks("taskId2", "taskName2", asList("taskTypeId"),
        ImmutableMap.of("prekey", asList("prevalue"), "key", asList("value")));
  }

  @Test(expected = IllegalStateException.class)
  public void testCheckForConflictingTasks_conflictingTaskMultipleConfigValues() {
    underTest.checkForConflictingTasks("taskId2", "taskName2", asList("taskTypeId"),
        ImmutableMap.of("key", asList("prevalue", "value")));
  }

  @Test(expected = IllegalStateException.class)
  public void testCheckForConflictingTasks_shouldCheckDatabaseForTaskState() {
    TaskResultState taskResultState = mock(TaskResultState.class);
    when(taskResultStateStore.getState(taskInfo)).thenReturn(Optional.of(taskResultState));
    when(taskResultState.getState()).thenReturn(RUNNING);

    underTest = new TaskUtils(() -> taskScheduler, taskResultStateStore);

    underTest.checkForConflictingTasks("taskId2", "taskName2", asList("taskTypeId"),
        ImmutableMap.of("key", asList("value")));

    verify(currentState, never()).getState();
    verify(taskResultStateStore).getState(taskInfo);
  }
}
