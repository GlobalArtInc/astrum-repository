
package org.globalart.astrum.blobstore;

import java.util.Collections;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStoreMetrics;

/**
 * An implementation of {@link BlobStoreMetrics} indicating the metrics can't be loaded.
 *
 * @since 3.19
 */
public class UnavailableBlobStoreMetrics
    implements BlobStoreMetrics
{
  private static final UnavailableBlobStoreMetrics INSTANCE = new UnavailableBlobStoreMetrics();

  private UnavailableBlobStoreMetrics() {
  }

  public static UnavailableBlobStoreMetrics getInstance() {
    return INSTANCE;
  }

  @Override
  public long getBlobCount() {
    return 0L;
  }

  @Override
  public long getTotalSize() {
    return 0L;
  }

  @Override
  public long getAvailableSpace() {
    return 0L;
  }

  @Override
  public boolean isUnlimited() {
    return false;
  }

  @Override
  public Map<String, Long> getAvailableSpaceByFileStore() {
    return Collections.emptyMap();
  }

  @Override
  public boolean isUnavailable() {
    return true;
  }

  @Override
  public String toString() {
    return "UnavailableBlobStoreMetrics{" +
        "blobCount=" + getBlobCount() +
        ", totalSize=" + getTotalSize() +
        ", availableSpace=" + getAvailableSpace() +
        ", unlimited=" + isUnlimited() +
        ", availableSpaceByFileStore=" + getAvailableSpaceByFileStore() +
        ", unavailable=" + isUnavailable() +
        '}';
  }
}
