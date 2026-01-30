
package org.globalart.astrum.repository.internal.blobstore;

import java.util.List;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaResult;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;

import com.codahale.metrics.health.HealthCheck.Result;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlobStoreQuotaHealthCheckTest
{
  @Mock
  private BlobStoreManager blobStoreManager;

  @Mock
  private BlobStore blobStore;

  @Mock
  private BlobStoreMetrics blobStoreMetrics;

  @Mock
  private BlobStoreConfiguration blobStoreConfiguration;

  @Mock
  private BlobStoreQuotaService quotaService;

  @Mock
  private BlobStoreQuotaResult quotaResult;

  private BlobStoreQuotaHealthCheck blobStoreHealthCheck =
      new BlobStoreQuotaHealthCheck(() -> blobStoreManager, () -> quotaService);

  @Before
  public void setup() {
    when(blobStore.getBlobStoreConfiguration()).thenReturn(blobStoreConfiguration);
    when(blobStoreConfiguration.getName()).thenReturn("test");
  }

  @Test
  public void testCheck_noneConfigured() {
    when(blobStoreManager.browse()).thenReturn(List.of());
    Result result = blobStoreHealthCheck.check();

    assertTrue(result.isHealthy());
  }

  @Test
  public void testCheck_noQuotaViolation() {
    when(blobStoreManager.browse()).thenReturn(List.of(blobStore));
    when(quotaService.checkQuota(blobStore)).thenReturn(quotaResult);
    when(quotaResult.isViolation()).thenReturn(false);

    Result result = blobStoreHealthCheck.check();

    assertTrue(result.isHealthy());

  }

  @Test
  public void testCheck_violatingQuota() {
    when(blobStoreManager.browse()).thenReturn(List.of(blobStore));
    when(quotaService.checkQuota(blobStore)).thenReturn(quotaResult);
    when(quotaResult.isViolation()).thenReturn(true);
    Result result = blobStoreHealthCheck.check();

    assertFalse(result.isHealthy());
  }

  @Test
  public void testCheck_noQuota() {
    when(blobStoreManager.browse()).thenReturn(List.of(blobStore));
    Result result = blobStoreHealthCheck.check();

    assertTrue(result.isHealthy());
  }
}
