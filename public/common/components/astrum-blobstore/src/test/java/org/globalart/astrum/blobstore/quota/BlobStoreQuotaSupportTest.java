
package org.globalart.astrum.blobstore.quota;

import org.junit.Test;
import org.mockito.Mock;
import org.slf4j.Logger;

import org.globalart.astrum.blobstore.MockBlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlobStoreQuotaSupportTest
{

  @Mock
  private BlobStore blobStore;

  @Mock
  private BlobStoreQuotaService quotaService;

  @Mock
  private Logger logger;

  @Test
  public void getLimitHandlesNumbersProperly() {
    BlobStoreConfiguration config = new MockBlobStoreConfiguration();
    config.attributes(BlobStoreQuotaSupport.ROOT_KEY).set(BlobStoreQuotaSupport.LIMIT_KEY, 1);
    assertThat(BlobStoreQuotaSupport.getLimit(config), is(1L));

    config.attributes(BlobStoreQuotaSupport.ROOT_KEY).set(BlobStoreQuotaSupport.LIMIT_KEY, 0);
    assertThat(BlobStoreQuotaSupport.getLimit(config), is(0L));

    config.attributes(BlobStoreQuotaSupport.ROOT_KEY).set(BlobStoreQuotaSupport.LIMIT_KEY, -1);
    assertThat(BlobStoreQuotaSupport.getLimit(config), is(-1L));
  }

  @Test(expected = IllegalArgumentException.class)
  public void getLimitHandlesErrorCases() {
    BlobStoreConfiguration config = new MockBlobStoreConfiguration();
    config.attributes(BlobStoreQuotaSupport.ROOT_KEY).set(BlobStoreQuotaSupport.LIMIT_KEY, null);
    BlobStoreQuotaSupport.getLimit(config);
  }

  @Test
  public void passedQuotaLogsNothing() {
    BlobStoreQuotaResult result = new BlobStoreQuotaResult(false, "name", "msg");
    when(quotaService.checkQuota(blobStore)).thenReturn(result);

    BlobStoreQuotaSupport.quotaCheckJob(blobStore, quotaService, logger);

    verify(logger, never()).error(anyString());
    verify(logger, never()).warn("msg");
  }

  @Test
  public void failedQuotaLogsResult() {
    BlobStoreQuotaResult result = new BlobStoreQuotaResult(true, "name", "msg");
    when(quotaService.checkQuota(blobStore)).thenReturn(result);

    BlobStoreQuotaSupport.quotaCheckJob(blobStore, quotaService, logger);

    verify(logger, never()).error(anyString());
    verify(logger).warn("msg");
  }

  @Test
  public void quotaCheckJobExceptionsAreCaught() {
    when(blobStore.getBlobStoreConfiguration()).thenReturn(mock(BlobStoreConfiguration.class));
    when(blobStore.getBlobStoreConfiguration().getName()).thenReturn("testConfig");
    doThrow(new RuntimeException()).when(quotaService).checkQuota(blobStore);

    BlobStoreQuotaSupport.quotaCheckJob(blobStore, quotaService, logger);

    verify(logger).error(anyString(), anyString(), any(RuntimeException.class));
    verify(logger, never()).warn(anyString());
  }
}
