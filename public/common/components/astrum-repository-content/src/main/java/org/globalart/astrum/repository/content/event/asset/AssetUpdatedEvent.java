
package org.globalart.astrum.repository.content.event.asset;

import org.globalart.astrum.repository.content.Asset;

/**
 * Event sent whenever an {@link Asset} is updated.
 *
 * @since 3.26
 */
public class AssetUpdatedEvent
    extends AssetEvent
{
  protected AssetUpdatedEvent(final Asset asset) {
    super(asset);
  }
}
