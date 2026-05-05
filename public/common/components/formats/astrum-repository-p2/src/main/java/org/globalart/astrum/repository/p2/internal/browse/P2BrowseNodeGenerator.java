
package org.globalart.astrum.repository.p2.internal.browse;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;
import org.globalart.astrum.repository.p2.internal.P2Format;

import org.springframework.beans.factory.annotation.Qualifier;

import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.appendPath;
import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.fromPaths;
import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.last;
import static org.globalart.astrum.repository.p2.internal.browse.P2BrowseNodeGeneratorHelper.computeComponentPath;
import static org.globalart.astrum.repository.p2.internal.browse.P2BrowseNodeGeneratorHelper.splitPath;

@org.springframework.stereotype.Component
@Qualifier(P2Format.NAME)
public class P2BrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    List<BrowsePath> browsePaths = new ArrayList<>(computeComponentPaths(asset));
    appendPath(browsePaths, last(splitPath(asset.path())), asset.path());
    return browsePaths;
  }

  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    return fromPaths(computeComponentPath(splitPath(asset.path()),
        asset.component().map(Component::name),
        asset.component().map(Component::version)), true);
  }
}
