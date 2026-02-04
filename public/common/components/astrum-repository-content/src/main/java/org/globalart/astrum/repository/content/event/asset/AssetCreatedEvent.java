
package org.globalart.astrum.repository.content.event.asset;

import org.globalart.astrum.repository.content.Asset;

/**
 * Event sent whenever an {@link Asset} is created.
 *
 * @since 3.26
 */
public class AssetCreatedEvent
    extends AssetEvent
{
  public AssetCreatedEvent(final Asset asset) {
    super(asset);
  }

  @Override
  public String toString() {
    return "AssetCreatedEvent{} " + super.toString();
  }
}
