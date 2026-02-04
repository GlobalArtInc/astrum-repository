
package org.globalart.astrum.repository.content.event.asset;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.content.store.InternalIds.contentRepositoryId;

/**
 * Base {@link Asset} event.
 *
 * @since 3.26
 */
public class AssetEvent
    extends ContentStoreEvent
{
  private final Asset asset;

  protected AssetEvent(final Asset asset) {
    super(contentRepositoryId(asset));
    this.asset = checkNotNull(asset);
  }

  public Asset getAsset() {
    return asset;
  }

  @Override
  public String toString() {
    return "AssetEvent{" +
        "asset=" + asset +
        "} " + super.toString();
  }
}
