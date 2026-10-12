package org.globalart.astrum.repository.swift.internal.browse;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;
import org.globalart.astrum.repository.swift.AssetKind;
import org.globalart.astrum.repository.swift.SwiftFormat;

import org.springframework.beans.factory.annotation.Qualifier;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.appendPath;
import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.fromPaths;

@org.springframework.stereotype.Component
@Qualifier(SwiftFormat.NAME)
public class SwiftBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    checkNotNull(asset);
    return asset.component().map(component -> {
      List<BrowsePath> paths = computeComponentPaths(asset);
      appendPath(paths, displayName(asset, component), asset.path());
      return paths;
    }).orElseGet(() -> super.computeAssetPaths(asset));
  }

  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    checkNotNull(asset);
    Component component = asset.component().get(); // NOSONAR: caller guarantees this
    List<String> segments = new ArrayList<>();
    segments.add(component.namespace());
    segments.add(component.name());
    segments.add(component.version());
    return fromPaths(segments, true);
  }

  static String displayName(final Asset asset, final Component component) {
    if (AssetKind.ARCHIVE.name().equals(asset.kind())) {
      return component.name() + "-" + component.version() + ".zip";
    }
    String path = asset.path();
    return path.substring(path.lastIndexOf('/') + 1);
  }
}
