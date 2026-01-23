
package org.globalart.astrum.blobstore.group.internal;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.common.math.Math2;

import static java.util.Collections.unmodifiableMap;

/**
 * An implementation of {@link BlobStoreMetrics} that combines metrics
 * from member metrics.
 *
 * @since 3.14
 */
public class BlobStoreGroupMetrics
    implements BlobStoreMetrics
{
  private final long blobCount;

  private final long totalSize;

  private final Map<String, Long> availableSpaceByFileStore;

  private final boolean unlimited;

  private final boolean unavailable;

  public BlobStoreGroupMetrics(final Iterable<BlobStoreMetrics> membersMetrics) {
    long aggregatedBlobCount = 0L;
    long aggregatedTotalSize = 0L;
    Map<String, Long> aggregatedAvailableSpaceByFileStore = new HashMap<>();
    boolean aggregatedUnlimited = false;
    int totalMembers = 0;
    int unavailableMembers = 0;

    for (BlobStoreMetrics memberMetrics : membersMetrics) {
      aggregatedBlobCount = Math2.addClamped(aggregatedBlobCount, memberMetrics.getBlobCount());
      aggregatedTotalSize = Math2.addClamped(aggregatedTotalSize, memberMetrics.getTotalSize());
      aggregatedAvailableSpaceByFileStore.putAll(memberMetrics.getAvailableSpaceByFileStore());
      aggregatedUnlimited = aggregatedUnlimited || memberMetrics.isUnlimited();
      totalMembers += 1;
      if (memberMetrics.isUnavailable()) {
        unavailableMembers += 1;
      }
    }

    this.blobCount = aggregatedBlobCount;
    this.totalSize = aggregatedTotalSize;
    this.availableSpaceByFileStore = unmodifiableMap(aggregatedAvailableSpaceByFileStore);
    this.unlimited = aggregatedUnlimited;
    this.unavailable = totalMembers > 0 && unavailableMembers == totalMembers;
  }

  @Override
  public long getBlobCount() {
    return blobCount;
  }

  @Override
  public long getTotalSize() {
    return totalSize;
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
    return unavailable;
  }

  public String toString() {
    return "BlobStoreGroupMetrics{" +
        "blobCount=" + blobCount +
        ", totalSize=" + totalSize +
        ", availableSpaceByFileStore=" + availableSpaceByFileStore +
        ", unlimited=" + unlimited +
        ", unavailable=" + unavailable +
        '}';
  }
}
