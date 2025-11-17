
package org.globalart.astrum.internal.security.secrets;

import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ReEncryptionRequiredHealthCheckTests
{
  @Mock
  private SecretsService secretsService;

  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private TaskInfo taskInfo;

  @InjectMocks
  private ReEncryptionRequiredHealthCheck underTest;

  @Test
  public void testHealthy() throws Exception {
    when(taskScheduler.getTaskByTypeId(anyString())).thenReturn(null);
    when(secretsService.isReEncryptRequired()).thenReturn(false);
    assertThat(underTest.check().isHealthy()).isTrue();

    // when task is running should return healthy
    when(taskScheduler.getTaskByTypeId(anyString())).thenReturn(taskInfo);
    when(secretsService.isReEncryptRequired()).thenReturn(true);
    assertThat(underTest.check().isHealthy()).isTrue();
  }

  @Test
  public void testUnhealthy() throws Exception {
    when(taskScheduler.getTaskByTypeId(anyString())).thenReturn(null);
    when(secretsService.isReEncryptRequired()).thenReturn(true);
    assertThat(underTest.check().isHealthy()).isFalse();
  }
}
