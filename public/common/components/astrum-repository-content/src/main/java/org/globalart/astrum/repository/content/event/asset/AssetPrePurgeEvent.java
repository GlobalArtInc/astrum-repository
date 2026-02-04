
package org.globalart.astrum.repository.content.event.asset;

import java.util.Arrays;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event sent just before a large number of {@link Asset}s without components are purged.
 *
 * @since 3.27
 */
public class AssetPrePurgeEvent
    extends ContentStoreEvent
{
  private final int[] assetIds;

  public AssetPrePurgeEvent(final int contentRepositoryId, final int[] assetIds) { // NOSONAR
    super(contentRepositoryId);
    this.assetIds = checkNotNull(assetIds);
  }

  public int[] getAssetIds() {
    return assetIds; // NOSONAR
  }

  @Override
  public String toString() {
    return "AssetPrePurgeEvent{" +
        "assetIds=" + Arrays.toString(assetIds) +
        "} " + super.toString();
  }
}
