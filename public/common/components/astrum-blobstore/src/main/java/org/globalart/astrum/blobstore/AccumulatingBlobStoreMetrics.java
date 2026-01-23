
package org.globalart.astrum.blobstore;

import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.common.math.Math2;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An implementation of {@link BlobStoreMetrics} that supports adding to the blobCount and totalSize fields.
 *
 * @since 3.2.1
 */
public class AccumulatingBlobStoreMetrics
    implements BlobStoreMetrics
{
  private long blobCount;

  private long totalSize;

  private final Map<String, Long> availableSpaceByFileStore;

  private final boolean unlimited;

  public AccumulatingBlobStoreMetrics(
      final long blobCount,
      final long totalSize,
      final Map<String, Long> availableSpaceByFileStore,
      final boolean unlimited)
  {
    this.blobCount = blobCount;
    this.totalSize = totalSize;
    this.availableSpaceByFileStore = checkNotNull(availableSpaceByFileStore);
    this.unlimited = unlimited;
  }

  @Override
  public long getBlobCount() {
    return blobCount;
  }

  public void addBlobCount(long blobCount) {
    this.blobCount += blobCount;
  }

  @Override
  public long getTotalSize() {
    return totalSize;
  }

  public void addTotalSize(long totalSize) {
    this.totalSize += totalSize;
  }

  @Override
  public long getAvailableSpace() {
    return availableSpaceByFileStore.values()
        .stream()
        .reduce(Math2::addClamped)
        .orElse(0L);
  }

  @Override
  public boolean isUnlimited() {
    return unlimited;
  }

  @Override
  public Map<String, Long> getAvailableSpaceByFileStore() {
    return availableSpaceByFileStore;
  }

  @Override
  public boolean isUnavailable() {
    return false;
  }

  public String toString() {
    return "AccumulatingBlobStoreMetrics{" +
        "blobCount=" + blobCount +
        ", totalSize=" + totalSize +
        ", availableSpaceByFileStore=" + availableSpaceByFileStore +
        ", unlimited=" + unlimited +
        '}';
  }
}
