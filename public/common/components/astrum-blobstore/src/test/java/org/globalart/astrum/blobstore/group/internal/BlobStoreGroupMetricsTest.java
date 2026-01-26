
package org.globalart.astrum.blobstore.group.internal;

import java.util.Arrays;

import org.globalart.astrum.blobstore.api.BlobStoreMetrics;

import org.junit.Test;

import static java.util.Collections.emptyList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class BlobStoreGroupMetricsTest
{

  @Test
  public void emptyMetricsIsAvailable() {
    assertThat(new BlobStoreGroupMetrics(emptyList()).isUnavailable(), is(false));
  }

  @Test
  public void metricsWithNoAvailableMemberIsUnavailable() {
    BlobStoreMetrics blobStoreMetrics = mock(BlobStoreMetrics.class);
    BlobStoreMetrics otherBlobStoreMetrics = mock(BlobStoreMetrics.class);
    when(blobStoreMetrics.isUnavailable()).thenReturn(true);
    when(otherBlobStoreMetrics.isUnavailable()).thenReturn(true);
    BlobStoreGroupMetrics groupMetrics =
        new BlobStoreGroupMetrics(Arrays.asList(blobStoreMetrics, otherBlobStoreMetrics));
    verify(blobStoreMetrics).isUnavailable();
    verify(blobStoreMetrics).getAvailableSpaceByFileStore();
    verify(otherBlobStoreMetrics).isUnavailable();
    verify(otherBlobStoreMetrics).getAvailableSpaceByFileStore();
    assertThat(groupMetrics.isUnavailable(), is(true));
  }

  @Test
  public void metricsWithOneAvailableMemberIsAvailable() {
    BlobStoreMetrics blobStoreMetrics = mock(BlobStoreMetrics.class);
    when(blobStoreMetrics.isUnavailable()).thenReturn(true);
    BlobStoreMetrics otherBlobStoreMetrics = mock(BlobStoreMetrics.class);
    BlobStoreGroupMetrics groupMetrics =
        new BlobStoreGroupMetrics(Arrays.asList(blobStoreMetrics, otherBlobStoreMetrics));
    verify(blobStoreMetrics).isUnavailable();
    verify(blobStoreMetrics).getAvailableSpaceByFileStore();
    verify(otherBlobStoreMetrics).isUnavailable();
    verify(otherBlobStoreMetrics).getAvailableSpaceByFileStore();
    assertThat(groupMetrics.isUnavailable(), is(false));
  }
}
