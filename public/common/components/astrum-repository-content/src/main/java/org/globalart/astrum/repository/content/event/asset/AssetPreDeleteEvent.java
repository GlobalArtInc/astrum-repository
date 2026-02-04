
package org.globalart.astrum.repository.content.event.asset;

import org.globalart.astrum.repository.content.Asset;

/**
 * Event sent just before an {@link Asset} is deleted.
 *
 * @since 3.27
 */
public class AssetPreDeleteEvent
    extends AssetEvent
{
  public AssetPreDeleteEvent(final Asset asset) {
    super(asset);
  }

  @Override
  public String toString() {
    return "AssetPreDeleteEvent{} " + super.toString();
  }
}
