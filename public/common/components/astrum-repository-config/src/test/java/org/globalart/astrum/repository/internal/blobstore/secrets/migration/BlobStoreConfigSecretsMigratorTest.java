
package org.globalart.astrum.repository.internal.blobstore.secrets.migration;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.internal.blobstore.secrets.migration.BlobStoreConfigSecretsMigrator.S3_TYPE;
import static org.globalart.astrum.repository.internal.blobstore.secrets.migration.BlobStoreConfigSecretsMigrator.secretKeys;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlobStoreConfigSecretsMigratorTest
{
  @Mock
  private BlobStoreManager blobStoreManager;

  @Mock
  private SecretsService secretsService;

  private BlobStoreConfigSecretsMigrator underTest;

  @Before
  public void setUp() {
    underTest = new BlobStoreConfigSecretsMigrator(blobStoreManager, secretsService);
    when(secretsService.from(anyString())).thenAnswer(invocation -> {
      Secret secret = mock(Secret.class);
      when(secret.getId()).thenReturn(invocation.getArgument(0));
      when(secret.decrypt()).thenReturn(Strings2.EMPTY.toCharArray());
      return secret;
    });
  }

  @Test
  public void testMigrateSecrets() throws Exception {
    mockBrowseCall(10, 4);

    underTest.migrate();

    verify(blobStoreManager).browse();
    verify(blobStoreManager, times(4)).update(any(BlobStoreConfiguration.class));
    verify(secretsService, times(8)).from(anyString());
  }

  @Test
  public void testMigrateNoSecrets() throws Exception {
    mockBrowseCall(5, 0);

    underTest.migrate();

    verify(blobStoreManager).browse();
    verify(blobStoreManager, never()).update(any(BlobStoreConfiguration.class));
    verify(secretsService, never()).from(anyString());
  }

  private void mockBrowseCall(final int totalSize, final int s3BlobStores) {
    List<BlobStore> results = new ArrayList<>();

    for (int i = 0; i < totalSize; i++) {
      BlobStore blobStore = mock(BlobStore.class);
      BlobStoreConfiguration blobStoreConfiguration = mock(BlobStoreConfiguration.class);
      when(blobStore.getBlobStoreConfiguration()).thenReturn(blobStoreConfiguration);
      when(blobStoreConfiguration.getName()).thenReturn("blobstore-" + i);
      when(blobStoreConfiguration.copy(anyString())).thenReturn(blobStoreConfiguration);

      if (i < s3BlobStores) {
        when(blobStoreConfiguration.getType()).thenReturn(S3_TYPE);
        Map<String, Map<String, Object>> attributes = new HashMap<>();
        Map<String, Object> config = new HashMap<>();
        config.put(secretKeys.get(0), "legacy-secret-" + i);
        config.put(secretKeys.get(1), "legacy-token" + i);
        attributes.put(S3_TYPE, config);

        when(blobStoreConfiguration.getAttributes()).thenReturn(attributes);
      }
      else {
        when(blobStoreConfiguration.getType()).thenReturn("other");
      }
      results.add(blobStore);
    }

    when(blobStoreManager.browse()).thenReturn(results);
  }
}
