
package org.globalart.astrum.repository.content.browse;

import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.browse.node.BrowsePathBuilder;
import org.globalart.astrum.repository.content.Asset;

import com.google.common.base.Splitter;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH_CHAR;

/**
 * Asset-led layout that assumes that components have the same path as their assets.
 *
 * @since 3.24
 */
public abstract class AssetPathBrowseNodeGenerator
    implements BrowseNodeGenerator
{
  @Override
  public boolean hasMultipleAssetsPerComponent() {
    return false;
  }

  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    checkNotNull(asset);
    List<String> pathSegments = Splitter.on(SLASH_CHAR).omitEmptyStrings().splitToList(asset.path());
    return BrowsePathBuilder.fromPaths(pathSegments, false);
  }

  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    return computeAssetPaths(asset);
  }
}
