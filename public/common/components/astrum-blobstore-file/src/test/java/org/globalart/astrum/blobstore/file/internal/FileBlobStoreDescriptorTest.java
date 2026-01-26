
package org.globalart.astrum.blobstore.file.internal;

import java.nio.file.Files;
import java.nio.file.Path;

import org.globalart.astrum.blobstore.BlobStoreUtil;
import org.globalart.astrum.blobstore.MockBlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.file.FileBlobStore.CONFIG_KEY;
import static org.globalart.astrum.blobstore.file.FileBlobStore.PATH_KEY;

@RunWith(MockitoJUnitRunner.Silent.class)
public class FileBlobStoreDescriptorTest

{
  @Mock
  BlobStoreQuotaService quotaService;

  @Mock
  BlobStoreUtil blobStoreUtil;

  @Mock
  ApplicationDirectories applicationDirectories;

  @Mock
  FileBlobStorePathValidator fileBlobStorePathValidator;

  FileBlobStoreDescriptor descriptor;

  private static final int MAX_NAME_LENGTH = 255;

  @Before
  public void setup() {
    descriptor =
        new FileBlobStoreDescriptor(quotaService, applicationDirectories, blobStoreUtil, fileBlobStorePathValidator);
    when(blobStoreUtil.validateFilePath(anyString(), anyInt())).thenReturn(true);
  }

  @Test
  public void descriptorValidationCallQuotaValidation() throws Exception {
    BlobStoreConfiguration config = new MockBlobStoreConfiguration();
    Path tempDir = Files.createTempDirectory("test");
    config.attributes(CONFIG_KEY).set(PATH_KEY, tempDir.toAbsolutePath().toString());
    descriptor.validateConfig(config);
    verify(quotaService, times(1)).validateSoftQuotaConfig(config);
  }

  @Test
  public void descriptorValidationCallPathValidation() throws Exception {
    BlobStoreConfiguration config = new MockBlobStoreConfiguration();
    String tempDir = Files.createTempDirectory("test").toString();
    config.attributes(CONFIG_KEY).set(PATH_KEY, tempDir);
    descriptor.validateConfig(config);
    verify(blobStoreUtil, times(1)).validateFilePath(tempDir, MAX_NAME_LENGTH);
  }

}
