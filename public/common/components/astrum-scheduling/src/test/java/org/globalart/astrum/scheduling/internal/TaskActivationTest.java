
package org.globalart.astrum.scheduling.internal;

import java.util.concurrent.Future;

import org.globalart.astrum.scheduling.CurrentState;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.spi.SchedulerSPI;

import org.junit.Before;
import org.junit.Test;
import org.mockito.InOrder;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Arrays.asList;
import static org.mockito.Mockito.doReturn;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.scheduling.TaskState.RUNNING;
import static org.globalart.astrum.scheduling.TaskState.WAITING;

@RunWith(MockitoJUnitRunner.Silent.class)
public class TaskActivationTest

{
  @Mock
  private SchedulerSPI schedulerSpi;

  @Mock
  private TaskInfo runningTask;

  @Mock
  private TaskInfo waitingTask;

  @Mock
  private CurrentState runningTaskCurrentState;

  @Mock
  private CurrentState waitingTaskCurrentState;

  @Mock
  private Future<?> runningTaskFuture;

  private TaskActivation underTest;

  @Before
  public void setUp() throws Exception {
    when(runningTask.getCurrentState()).thenReturn(runningTaskCurrentState);
    when(runningTaskCurrentState.getState()).thenReturn(RUNNING);
    doReturn(runningTaskFuture).when(runningTaskCurrentState).getFuture();

    when(waitingTask.getCurrentState()).thenReturn(waitingTaskCurrentState);
    when(waitingTaskCurrentState.getState()).thenReturn(WAITING);
    doReturn(null).when(waitingTaskCurrentState).getFuture();

    when(schedulerSpi.listsTasks()).thenReturn(asList(runningTask, waitingTask));
    underTest = new TaskActivation(schedulerSpi);
    underTest.start();
  }

  /*
   * pause scheduler and cancel tasks on freeze
   */
  @Test
  public void testPauseScheduler_cancelTasksOnFreeze() {
    InOrder inOrder = inOrder(schedulerSpi, runningTaskFuture);

    underTest.freeze();
    inOrder.verify(schedulerSpi).pause();
    inOrder.verify(runningTaskFuture).cancel(false);

    inOrder.verifyNoMoreInteractions();
  }

  /*
   * restart scheduler when database on unfreeze
   */
  @Test
  public void testRestartScheduler_onUnfreeze() throws Exception {
    InOrder inOrder = inOrder(schedulerSpi, runningTaskFuture);

    underTest.stop();
    inOrder.verify(schedulerSpi).pause();
    underTest.freeze();
    underTest.start();
    inOrder.verify(schedulerSpi, times(0)).resume();
    underTest.unfreeze();
    inOrder.verify(schedulerSpi).resume();

    inOrder.verifyNoMoreInteractions();
  }

  /*
   * scheduler not resumed on startup if frozen
   */
  @Test
  public void testSchedulerNotResumed_onFrozenStartup() throws Exception {
    InOrder inOrder = inOrder(schedulerSpi, runningTaskFuture);

    underTest.stop();
    inOrder.verify(schedulerSpi).pause();
    underTest.freeze();
    underTest.start();

    inOrder.verifyNoMoreInteractions();
  }
}
