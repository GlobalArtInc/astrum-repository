package org.globalart.astrum.repository.huggingface.internal.browse;

import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.browse.node.BrowsePathBuilder;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.AssetPathBrowseNodeGenerator;
import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;

import com.google.common.base.Splitter;
import org.springframework.beans.factory.annotation.Qualifier;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Files are browsed as {@code <repo type>/<repo id>/<commit>/<file path>}, the component being the commit folder;
 * the information of a revision lies next to the commit folders as {@code <revision>.json}.
 */
@org.springframework.stereotype.Component
@Qualifier(HuggingFaceFormat.NAME)
public class HuggingFaceBrowseNodeGenerator
    extends AssetPathBrowseNodeGenerator
{
  @Override
  public boolean hasMultipleAssetsPerComponent() {
    return true;
  }

  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    checkNotNull(asset);
    List<BrowsePath> assetPaths = computeAssetPaths(asset);
    int depth = asset.component().map(HuggingFaceBrowseNodeGenerator::componentDepth).orElse(assetPaths.size() - 1);
    if (depth <= 0 || depth >= assetPaths.size()) {
      depth = assetPaths.size() - 1;
    }
    return BrowsePathBuilder.fromPaths(assetPaths.subList(0, depth).stream().map(BrowsePath::getDisplayName).toList(),
        true);
  }

  private static int componentDepth(final Component component) {
    return 1 + Splitter.on('/').omitEmptyStrings().splitToList(component.name()).size() + 1;
  }
}
