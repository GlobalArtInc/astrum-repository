
package org.globalart.astrum.repository.content.event.asset;

import org.globalart.astrum.repository.content.Asset;

/**
 * Event sent whenever an asset's path is changed.
 *
 * @since 3.26
 */
public class AssetPathChangedEvent
    extends AssetUpdatedEvent
{
  private final String oldPath;

  private final String newPath;

  public AssetPathChangedEvent(final Asset asset, final String oldPath, final String newPath) {
    super(asset);
    this.oldPath = oldPath;
    this.newPath = newPath;
  }

  public String getOldPath() {
    return oldPath;
  }

  public String getNewPath() {
    return newPath;
  }

  @Override
  public String toString() {
    return "AssetPathChangedEvent{" +
        "oldPath='" + oldPath + '\'' +
        ", newPath='" + newPath + '\'' +
        "} " + super.toString();
  }
}
