
package org.globalart.astrum.internal.apikey;

import java.util.Optional;
import java.util.UUID;

import org.globalart.astrum.common.cooperation2.Cooperation2Factory;
import org.globalart.astrum.common.cooperation2.datastore.DefaultCooperation2Factory;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.crypto.apikey.ApiKeysReEncryptService;
import org.globalart.astrum.crypto.secrets.EncryptionKeyValidator;
import org.globalart.astrum.crypto.secrets.ReEncryptionNotSupportedException;
import org.globalart.astrum.internal.security.apikey.task.ReEncryptPrincipalsTaskDescriptor;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.security.UserIdHelper;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.mockito.Spy;

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ApiKeysReEncryptServiceImplTest
{
  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private EncryptionKeyValidator encryptionKeyValidator;

  @Spy
  private TaskConfiguration configuration;

  @Spy
  private TaskInfo taskInfo;

  @Mock
  private DatabaseCheck databaseCheck;

  @Captor
  private ArgumentCaptor<TaskConfiguration> taskConfigurationCaptor;

  private final MockedStatic<UserIdHelper> userIdHelper = mockStatic(UserIdHelper.class);

  private final Cooperation2Factory cooperationFactory = new DefaultCooperation2Factory();

  private ApiKeysReEncryptService underTest;

  @Before
  public void setup() {
    userIdHelper.when(UserIdHelper::get).thenReturn("test");
    when(databaseCheck.isAtLeast(anyString())).thenReturn(true);
    when(encryptionKeyValidator.getActiveKeyId()).thenReturn(Optional.of("old-key-id"));

    underTest = new ApiKeysReEncryptServiceImpl(taskScheduler,
        databaseCheck, cooperationFactory, true);
  }

  @After
  public void teardown() {
    userIdHelper.close();
  }

  @Test
  public void testSubmitReEncryption() {
    String taskId = UUID.randomUUID().toString();
    when(taskInfo.getId()).thenReturn(taskId);
    when(taskInfo.getConfiguration()).thenReturn(configuration);
    when(taskScheduler.submit(configuration)).thenReturn(taskInfo);
    when(taskScheduler.createTaskConfigurationInstance(ReEncryptPrincipalsTaskDescriptor.TYPE_ID))
        .thenReturn(configuration);
    when(taskScheduler.getTaskById(ReEncryptPrincipalsTaskDescriptor.TYPE_ID)).thenReturn(null).thenReturn(taskInfo);

    underTest.submitReEncryption("algorithmToBeUsed", null, null);

    verify(taskScheduler).submit(taskConfigurationCaptor.capture());
    TaskConfiguration submittedTask = taskConfigurationCaptor.getValue();
    assertThat(submittedTask).isNotNull();
    assertThat(submittedTask.getAlertEmail()).isNull();
    verify(submittedTask).setString("algorithmForDecryption", "algorithmToBeUsed");
  }

  @Test
  public void testSubmitReEncryption_ScheduledTaskWithNoParams() {
    String taskId = UUID.randomUUID().toString();
    when(taskInfo.getId()).thenReturn(taskId);
    when(taskInfo.getConfiguration()).thenReturn(configuration);
    when(taskScheduler.submit(configuration)).thenReturn(taskInfo);
    when(taskScheduler.createTaskConfigurationInstance(ReEncryptPrincipalsTaskDescriptor.TYPE_ID))
        .thenReturn(configuration);
    when(taskScheduler.getTaskById(ReEncryptPrincipalsTaskDescriptor.TYPE_ID)).thenReturn(null).thenReturn(taskInfo);

    underTest.submitReEncryption(null, null, null);
    verify(taskScheduler).submit(taskConfigurationCaptor.capture());

    TaskConfiguration submittedTask = taskConfigurationCaptor.getValue();
    assertThat(submittedTask).isNotNull();
    assertThat(submittedTask.getAlertEmail()).isNull();
    verify(submittedTask).setString("algorithmForDecryption", null);
  }

  @Test
  public void testSubmitReEncryption_ScheduledTaskWithEmailOnValidKey() {
    String notifyEmail = "notify@test.com";

    String taskId = UUID.randomUUID().toString();
    when(taskInfo.getId()).thenReturn(taskId);
    when(taskInfo.getConfiguration()).thenReturn(configuration);
    when(taskScheduler.submit(configuration)).thenReturn(taskInfo);
    when(taskScheduler.createTaskConfigurationInstance(ReEncryptPrincipalsTaskDescriptor.TYPE_ID))
        .thenReturn(configuration);
    when(taskScheduler.getTaskById(ReEncryptPrincipalsTaskDescriptor.TYPE_ID)).thenReturn(null).thenReturn(taskInfo);

    underTest.submitReEncryption("algorithmToBeUsed", null, notifyEmail);
    verify(taskScheduler).submit(taskConfigurationCaptor.capture());

    TaskConfiguration submittedTask = taskConfigurationCaptor.getValue();
    assertThat(submittedTask).isNotNull();
    verify(submittedTask).setAlertEmail(notifyEmail);
    verify(submittedTask).setString("algorithmForDecryption", "algorithmToBeUsed");
  }

  @Test
  public void testSubmitReEncryption_OnUnsupportedVersion() {
    when(databaseCheck.isAtLeast(anyString())).thenReturn(false);
    assertThrows(ReEncryptionNotSupportedException.class,
        () -> underTest.submitReEncryption("algorithmToBeUsed", null, null));
  }

  @Test
  public void testSubmitReEncryption_TaskAlreadySubmitted() {
    when(taskScheduler.getTaskByTypeId(ReEncryptPrincipalsTaskDescriptor.TYPE_ID)).thenReturn(taskInfo);
    assertThrows(IllegalStateException.class, () -> underTest.submitReEncryption(null, null, null));
  }
}
