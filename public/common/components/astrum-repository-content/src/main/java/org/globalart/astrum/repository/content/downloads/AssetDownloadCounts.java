package org.globalart.astrum.repository.content.downloads;

import java.util.Map;

import org.globalart.astrum.repository.content.Asset;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetId;

/**
 * Download counts of a set of assets of one format, as read by {@link AssetDownloadCounter}.
 */
public final class AssetDownloadCounts
{
  private final Map<Integer, Long> countsByAssetId;

  /**
   * @param countsByAssetId download counts keyed by internal asset id
   */
  public AssetDownloadCounts(final Map<Integer, Long> countsByAssetId) {
    this.countsByAssetId = checkNotNull(countsByAssetId);
  }

  /**
   * Returns how often the given asset was downloaded; {@code 0} when it never was or was not part of the lookup.
   */
  public long of(final Asset asset) {
    return countsByAssetId.getOrDefault(internalAssetId(asset), 0L);
  }

  /**
   * Returns the sum of the downloads of all assets of the lookup.
   */
  public long total() {
    return countsByAssetId.values().stream().mapToLong(Long::longValue).sum();
  }

  @Override
  public String toString() {
    return "AssetDownloadCounts" + countsByAssetId;
  }
}
