
package org.globalart.astrum.repository.content.event.asset;

import org.globalart.astrum.repository.content.Asset;

/**
 * Event sent whenever an {@link Asset}'s LastDownloaded time changes.
 *
 * @since 3.26
 */
public class AssetDownloadedEvent
    extends AssetUpdatedEvent
{
  public AssetDownloadedEvent(final Asset asset) {
    super(asset);
  }

  @Override
  public String toString() {
    return "AssetDownloadedEvent{} " + super.toString();
  }
}
