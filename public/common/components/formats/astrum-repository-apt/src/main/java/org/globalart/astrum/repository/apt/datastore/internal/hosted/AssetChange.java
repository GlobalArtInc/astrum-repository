
package org.globalart.astrum.repository.apt.datastore.internal.hosted;

import org.globalart.astrum.repository.apt.internal.hosted.AssetAction;
import org.globalart.astrum.repository.content.fluent.FluentAsset;

/**
 * Helper class which helps to connect asset and action performed under it
 *
 * @since 3.31
 */
public class AssetChange
{
  private final AssetAction action;

  private final FluentAsset asset;

  public AssetChange(final AssetAction action, final FluentAsset asset) {
    this.action = action;
    this.asset = asset;
  }

  public AssetAction getAction() {
    return action;
  }

  public FluentAsset getAsset() {
    return asset;
  }
}
