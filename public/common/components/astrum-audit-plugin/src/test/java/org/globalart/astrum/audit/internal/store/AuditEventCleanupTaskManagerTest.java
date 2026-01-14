
package org.globalart.astrum.audit.internal.store;

import java.util.Collections;
import java.util.List;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Mockito;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.schedule.Cron;
import org.globalart.astrum.scheduling.schedule.ScheduleFactory;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Unit tests for {@link AuditEventCleanupTaskManager}. No database or scheduler is invoked;
 * {@link TaskScheduler} is mocked.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class AuditEventCleanupTaskManagerTest
{
  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private TaskConfiguration taskConfiguration;

  @Mock
  private ScheduleFactory scheduleFactory;

  @Mock
  private Cron schedule;

  private AuditEventCleanupTaskManager manager;

  @BeforeEach
  void setUp() {
    when(taskScheduler.getScheduleFactory()).thenReturn(scheduleFactory);
    when(taskScheduler.createTaskConfigurationInstance(AuditEventCleanupTaskDescriptor.TYPE_ID))
        .thenReturn(taskConfiguration);
    when(scheduleFactory.cron(any(), any())).thenReturn(schedule);
    when(taskScheduler.listsTasks()).thenReturn(Collections.emptyList());
    manager = new AuditEventCleanupTaskManager(taskScheduler, "0 30 3 * * ?", true);
  }

  @Test
  void createsTaskWhenNoneExists() throws Exception {
    when(taskScheduler.getTaskByTypeId(AuditEventCleanupTaskDescriptor.TYPE_ID)).thenReturn(null);

    manager.doStart();

    verify(taskConfiguration).setName(AuditEventCleanupTaskDescriptor.TASK_NAME);
    verify(taskScheduler).scheduleTask(eq(taskConfiguration), eq(schedule));
  }

  @Test
  void preservesExistingTaskSchedule() throws Exception {
    TaskInfo existing = Mockito.mock(TaskInfo.class);
    when(taskScheduler.getTaskByTypeId(AuditEventCleanupTaskDescriptor.TYPE_ID)).thenReturn(existing);

    manager.doStart();

    verify(taskScheduler, never()).scheduleTask(any(), any());
  }

  @Test
  void doesNotRegisterWhenDisabledByFeatureFlag() throws Exception {
    AuditEventCleanupTaskManager disabled =
        new AuditEventCleanupTaskManager(taskScheduler, "0 30 3 * * ?", false);

    disabled.doStart();

    verify(taskScheduler, never()).scheduleTask(any(), any());
    verify(taskScheduler, never()).getTaskByTypeId(any());
  }

  @Test
  void removesDuplicateTasks() throws Exception {
    TaskInfo t1 = Mockito.mock(TaskInfo.class);
    TaskInfo t2 = Mockito.mock(TaskInfo.class);
    TaskConfiguration cfg = Mockito.mock(TaskConfiguration.class);
    when(cfg.getTypeId()).thenReturn(AuditEventCleanupTaskDescriptor.TYPE_ID);
    when(t1.getConfiguration()).thenReturn(cfg);
    when(t2.getConfiguration()).thenReturn(cfg);
    when(t1.getId()).thenReturn("aaa");
    when(t2.getId()).thenReturn("bbb");
    when(taskScheduler.listsTasks()).thenReturn(List.of(t1, t2));
    when(taskScheduler.getTaskByTypeId(AuditEventCleanupTaskDescriptor.TYPE_ID)).thenReturn(t1);

    manager.doStart();

    verify(t2).remove();
    verify(t1, never()).remove();
  }
}
