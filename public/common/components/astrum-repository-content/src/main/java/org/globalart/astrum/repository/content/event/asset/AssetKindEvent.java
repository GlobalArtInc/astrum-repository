
package org.globalart.astrum.repository.content.event.asset;

import org.globalart.astrum.repository.content.Asset;

/**
 * Event sent whenever an {@link Asset}'s kind is updated.
 *
 * @since 3.26
 */
public class AssetKindEvent
    extends AssetUpdatedEvent
{
  public AssetKindEvent(final Asset asset) {
    super(asset);
  }

  @Override
  public String toString() {
    return "AssetKindEvent{} " + super.toString();
  }
}
