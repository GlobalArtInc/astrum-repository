
package org.globalart.astrum.internal.scheduling;

import org.apache.commons.mail2.core.EmailException;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import com.google.common.collect.ImmutableMap;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskNotificationCondition;
import org.globalart.astrum.scheduling.TaskNotificationMessageGenerator;
import org.globalart.astrum.scheduling.events.TaskEventStoppedDone;
import org.globalart.astrum.scheduling.events.TaskEventStoppedFailed;

import java.util.List;

import static org.mockito.ArgumentMatchers.isNotNull;
import static org.mockito.Mockito.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests {@link AstrumTaskNotificationEmailSender}
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class AstrumTaskNotificationEmailSenderTest
{

  @Mock
  private EmailManager emailManager;

  @Mock
  private TaskNotificationMessageGenerator defaultTaskNotificationMessageGenerator;

  @Mock
  private TaskNotificationMessageGenerator customTaskNotificationMessageGenerator;

  private AstrumTaskNotificationEmailSender underTest;

  private MockedStatic<QualifierUtil> mockedStatic;

  @Before
  public void setup() {
    mockedStatic = mockStatic(QualifierUtil.class);
    when(defaultTaskNotificationMessageGenerator.completed(isNotNull())).thenReturn("completed message");
    when(defaultTaskNotificationMessageGenerator.failed(isNotNull(), isNotNull())).thenReturn("failure message");
    when(emailManager.constructMessage("completed message")).thenReturn("completed message");
    when(emailManager.constructMessage("failure message")).thenReturn("failure message");
    when(QualifierUtil.buildQualifierBeanMap(anyList())).thenReturn(
        ImmutableMap.of("DEFAULT", defaultTaskNotificationMessageGenerator, "CUSTOM",
            customTaskNotificationMessageGenerator));
    underTest = new AstrumTaskNotificationEmailSender(() -> emailManager,
        List.of(defaultTaskNotificationMessageGenerator, customTaskNotificationMessageGenerator));
  }

  @After
  public void teardown() {
    mockedStatic.close();
  }

  @Test
  public void generatesEmailIfTaskFailed() throws EmailException {
    TaskInfo taskInfo = mock(TaskInfo.class);
    TaskConfiguration taskConfiguration = mock(TaskConfiguration.class);
    when(taskInfo.getId()).thenReturn("taskId");
    when(taskInfo.getName()).thenReturn("test task");
    when(taskInfo.getConfiguration()).thenReturn(taskConfiguration);
    when(taskConfiguration.getAlertEmail()).thenReturn("foo@example.com");
    TaskEventStoppedFailed event = new TaskEventStoppedFailed(taskInfo, new RuntimeException());

    underTest.on(event);

    verify(emailManager).send(isNotNull());
  }

  @Test
  public void generatesNoEmailIfTaskCompletesAndConfigurationConditionIsFailedOnly() throws EmailException {
    TaskInfo taskInfo = mock(TaskInfo.class);
    TaskConfiguration taskConfiguration = mock(TaskConfiguration.class);
    when(taskInfo.getId()).thenReturn("taskId");
    when(taskInfo.getName()).thenReturn("test task");
    when(taskInfo.getConfiguration()).thenReturn(taskConfiguration);
    when(taskConfiguration.getAlertEmail()).thenReturn("foo@example.com");
    when(taskConfiguration.getNotificationCondition()).thenReturn(TaskNotificationCondition.FAILURE);
    TaskEventStoppedDone event = new TaskEventStoppedDone(taskInfo);

    underTest.on(event);

    verify(emailManager, never()).send(isNotNull());
  }

  @Test
  public void generatesEmailIfTaskCompletesAndConfigurationConditionIsCompleted() throws EmailException {
    TaskInfo taskInfo = mock(TaskInfo.class);
    TaskConfiguration taskConfiguration = mock(TaskConfiguration.class);
    when(taskInfo.getId()).thenReturn("taskId");
    when(taskInfo.getName()).thenReturn("test task");
    when(taskInfo.getConfiguration()).thenReturn(taskConfiguration);
    when(taskConfiguration.getAlertEmail()).thenReturn("foo@example.com");
    when(taskConfiguration.getNotificationCondition()).thenReturn(TaskNotificationCondition.SUCCESS_FAILURE);
    TaskEventStoppedDone event = new TaskEventStoppedDone(taskInfo);

    underTest.on(event);

    verify(emailManager).send(isNotNull());
  }

  @Test
  public void usesCustomMessageGeneratorIfAvailableForTaskType() {
    TaskInfo taskInfo = mock(TaskInfo.class);
    TaskConfiguration taskConfiguration = mock(TaskConfiguration.class);
    when(taskInfo.getId()).thenReturn("taskId");
    when(taskInfo.getTypeId()).thenReturn("CUSTOM");
    when(taskInfo.getName()).thenReturn("test task");
    when(taskInfo.getConfiguration()).thenReturn(taskConfiguration);
    when(taskConfiguration.getAlertEmail()).thenReturn("foo@example.com");
    when(taskConfiguration.getNotificationCondition()).thenReturn(TaskNotificationCondition.SUCCESS_FAILURE);
    when(customTaskNotificationMessageGenerator.completed(isNotNull())).thenReturn("body");
    TaskEventStoppedDone event = new TaskEventStoppedDone(taskInfo);

    underTest.on(event);

    verify(customTaskNotificationMessageGenerator).completed(isNotNull());
  }
}
