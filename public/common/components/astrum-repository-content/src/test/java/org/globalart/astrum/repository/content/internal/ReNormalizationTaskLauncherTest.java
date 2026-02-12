
package org.globalart.astrum.repository.content.internal;

import java.util.Optional;

import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.repository.content.tasks.normalize.ReNormalizeComponentVersionTaskDescriptor;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.content.tasks.normalize.ReNormalizeComponentVersionTask.CURSOR_KEY;
import static org.globalart.astrum.repository.content.tasks.normalize.ReNormalizeComponentVersionTask.GATE_KEY_FORMAT;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ReNormalizationTaskLauncherTest
{
  @Mock
  private GlobalKeyValueStore globalKeyValueStore;

  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private TaskConfiguration taskConfiguration;

  private ReNormalizationTaskLauncher underTest;

  @Before
  public void setUp() {
    underTest = new ReNormalizationTaskLauncher(globalKeyValueStore, taskScheduler);
    when(globalKeyValueStore.getKey(CURSOR_KEY)).thenReturn(Optional.empty());
    when(taskScheduler.createTaskConfigurationInstance(ReNormalizeComponentVersionTaskDescriptor.TYPE_ID))
        .thenReturn(taskConfiguration);
  }

  @Test
  public void testNoSubmissionWhenNoGateRaisedAndNoCursorPresent() throws Exception {
    underTest.doStart();

    verify(taskScheduler, never()).submit(taskConfiguration);
  }

  @Test
  public void testSubmitsDirectlyWhenAnyTargetFormatGateIsRaised() throws Exception {
    mockGateRaised("npm", true);

    underTest.doStart();

    verify(taskScheduler).createTaskConfigurationInstance(eq(ReNormalizeComponentVersionTaskDescriptor.TYPE_ID));
    verify(taskScheduler).submit(taskConfiguration);
  }

  @Test
  public void testSubmitsDirectlyWhenCursorIsPresentEvenIfNoGateIsRaised() throws Exception {
    mockCursorPresent();

    underTest.doStart();

    verify(taskScheduler).submit(taskConfiguration);
  }

  @Test
  public void testDoesNotDoubleScheduleWhenTaskAlreadyScheduledOrRunning() throws Exception {
    mockCursorPresent();
    when(taskScheduler.getTaskByTypeId(ReNormalizeComponentVersionTaskDescriptor.TYPE_ID))
        .thenReturn(mock(TaskInfo.class));

    underTest.doStart();

    verify(taskScheduler, never()).submit(taskConfiguration);
  }

  @Test
  public void testNeverInspectsFormatsOutsideRenormalizeFormats() throws Exception {
    underTest.doStart();

    verify(globalKeyValueStore, never()).getKey(GATE_KEY_FORMAT.formatted("maven2"));
    verify(globalKeyValueStore, never()).getKey(GATE_KEY_FORMAT.formatted("docker"));
  }

  @Test
  public void testSubmissionFailureIsSwallowedNotPropagated() throws Exception {
    mockCursorPresent();
    when(taskScheduler.submit(taskConfiguration)).thenThrow(new IllegalStateException("scheduler not started"));

    underTest.doStart();
  }

  private void mockGateRaised(final String format, final boolean raised) {
    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey(GATE_KEY_FORMAT.formatted(format));
    kv.setType(ValueType.BOOLEAN);
    kv.setValue(raised);
    when(globalKeyValueStore.getKey(GATE_KEY_FORMAT.formatted(format))).thenReturn(Optional.of(kv));
  }

  private void mockCursorPresent() {
    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey(CURSOR_KEY);
    kv.setType(ValueType.BOOLEAN);
    kv.setValue(true);
    when(globalKeyValueStore.getKey(CURSOR_KEY)).thenReturn(Optional.of(kv));
  }
}
