
package org.globalart.astrum.repository.apt.datastore.internal.browse;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.browse.node.BrowsePathBuilder;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.DefaultBrowseNodeGenerator;

import com.google.common.base.Splitter;
import org.springframework.beans.factory.annotation.Qualifier;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH;

/**
 * @since 3.31
 */
@org.springframework.stereotype.Component
@Qualifier(AptFormat.NAME)
public class AptBrowseNodeGenerator
    extends DefaultBrowseNodeGenerator
{
  private static final String PACKAGES_PATH = "packages";

  private static final String METADATA_PATH = "metadata";

  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    Component component = asset.component().orElse(null);
    if (component != null) {
      List<BrowsePath> path = computeComponentPaths(asset);
      BrowsePathBuilder.appendPath(path, asset.path().substring(1));
      return path;
    }

    List<String> pathParts = new ArrayList<>();
    String path = asset.path();
    if (path.endsWith(".deb") || path.endsWith(".udeb")) {
      pathParts.add(PACKAGES_PATH);
    }
    else if (!path.startsWith("/snapshots")) {
      pathParts.add(METADATA_PATH);
    }

    pathParts.addAll(Splitter.on(SLASH).omitEmptyStrings().splitToList(path));

    return BrowsePathBuilder.fromPaths(pathParts, true);
  }

  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    checkNotNull(asset);

    checkArgument(asset.component().isPresent());
    Component component = asset.component().get();

    List<String> pathParts = new ArrayList<>();
    pathParts.add(PACKAGES_PATH);
    pathParts.add(component.name().substring(0, 1).toLowerCase());
    pathParts.add(component.name());
    pathParts.add(component.version());
    pathParts.add(component.namespace());
    pathParts.add(component.name());
    return BrowsePathBuilder.fromPaths(pathParts, true);
  }
}
