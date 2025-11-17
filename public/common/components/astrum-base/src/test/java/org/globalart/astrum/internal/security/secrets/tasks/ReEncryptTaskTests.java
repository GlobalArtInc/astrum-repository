
package org.globalart.astrum.internal.security.secrets.tasks;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.UUID;

import org.globalart.astrum.crypto.secrets.SecretData;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.crypto.secrets.SecretsStore;
import org.globalart.astrum.scheduling.TaskConfiguration;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mock;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.doReturn;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ReEncryptTaskTests
{
  private static final String TEST_KEY_ID = "test-key";

  @Mock
  private SecretsService secretsService;

  @Mock
  private SecretsStore secretsStore;

  @Mock
  private TaskConfiguration taskConfiguration;

  private final Random random = new Random();

  private ReEncryptTask underTest;

  @Before
  public void setup() {
    underTest = spy(new ReEncryptTask(secretsService, secretsStore, 1));
    setupTaskConfig();
    doReturn(taskConfiguration).when(underTest).taskConfiguration();
  }

  @Test
  public void testReEncryptWorksAsExpected() throws Exception {
    String keyId = "old-key";

    List<SecretData> page1 = getMockSecretList(2, keyId);
    List<SecretData> page2 = getMockSecretList(4, keyId);
    List<SecretData> page3 = getMockSecretList(0, keyId);
    when(secretsStore.existWithDifferentKeyId(anyString())).thenReturn(true);
    when(secretsStore.fetchWithDifferentKeyId(anyString(), anyInt()))
        .thenReturn(page1)
        .thenReturn(page2)
        .thenReturn(page3);

    assertThat(underTest.execute()).isEqualTo(6);
    verify(secretsStore, times(3)).fetchWithDifferentKeyId(eq(TEST_KEY_ID), anyInt());
    verify(secretsService, times(6)).reEncrypt(any(SecretData.class), eq(TEST_KEY_ID));
  }

  @Test
  public void testReEncryptEmptyPage() throws Exception {
    String keyId = "old-key";

    List<SecretData> page1 = getMockSecretList(0, keyId);
    when(secretsStore.fetchWithDifferentKeyId(anyString(), anyInt())).thenReturn(page1);

    assertThat(underTest.execute()).isEqualTo(0);
    verifyNoMoreInteractions(secretsStore);
    verifyNoInteractions(secretsService);
  }

  private void setupTaskConfig() {
    taskConfiguration = new TaskConfiguration();
    taskConfiguration.setId(UUID.randomUUID().toString());
    taskConfiguration.setTypeId(ReEncryptTaskDescriptor.TYPE_ID);
    taskConfiguration.setString("keyId", TEST_KEY_ID);
  }

  private SecretData getMockSecretData(final int id, final String keyId, final String secret) {
    SecretData mockData = new SecretData();
    mockData.setId(id);
    mockData.setKeyId(keyId);
    mockData.setSecret(secret);
    return mockData;
  }

  private List<SecretData> getMockSecretList(final int size, String keyId) {
    List<SecretData> secretList = new ArrayList<>();
    for (int i = 0; i < size; i++) {
      int id = random.nextInt();
      String secret = "secret" + id;
      secretList.add(getMockSecretData(id, keyId, secret));
    }
    return secretList;
  }

}
