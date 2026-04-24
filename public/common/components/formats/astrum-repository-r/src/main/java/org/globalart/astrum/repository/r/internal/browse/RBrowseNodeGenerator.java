
package org.globalart.astrum.repository.r.internal.browse;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.browse.node.BrowsePathBuilder;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;
import org.globalart.astrum.repository.r.RFormat;

import com.google.common.base.Splitter;
import org.springframework.beans.factory.annotation.Qualifier;

import static com.google.common.base.Preconditions.checkNotNull;

@org.springframework.stereotype.Component
@Qualifier(RFormat.NAME)
public class RBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    checkNotNull(asset);
    if (asset.component().isPresent()) {
      List<BrowsePath> paths = computeComponentPaths(asset);
      BrowsePathBuilder.appendPath(paths, lastSegment(asset.path()), asset.path());
      return paths;
    }
    return BrowsePathBuilder.fromPaths(segments(asset.path()), false);
  }

  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    checkNotNull(asset);
    Optional<Component> component = asset.component();
    List<String> segments = segments(asset.path());
    if (component.isEmpty()) {
      return BrowsePathBuilder.fromPaths(segments.subList(0, segments.size() - 1), true);
    }
    List<String> paths = new ArrayList<>(segments.subList(0, segments.size() - 1));
    if (paths.isEmpty() || !paths.get(paths.size() - 1).equals(component.get().name())) {
      paths.add(component.get().name());
    }
    paths.add(component.get().version());
    return BrowsePathBuilder.fromPaths(paths, true);
  }

  private static List<String> segments(final String path) {
    return Splitter.on('/').omitEmptyStrings().splitToList(path);
  }
}
