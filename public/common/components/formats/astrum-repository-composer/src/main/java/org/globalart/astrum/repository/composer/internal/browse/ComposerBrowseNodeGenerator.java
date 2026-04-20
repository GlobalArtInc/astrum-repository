
package org.globalart.astrum.repository.composer.internal.browse;

import org.springframework.beans.factory.annotation.Qualifier;


import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;

import java.util.ArrayList;
import java.util.List;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.text.Strings2.isBlank;
import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.appendPath;
import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.fromPaths;

/**
 * Browse node generator for Composer.
 */
@org.springframework.stereotype.Component
@Qualifier(ComposerFormat.NAME)
public class ComposerBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator {

  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    checkNotNull(asset);

    return asset.component().map(component -> {

      // place asset under component, but use its true path as the request path for permission checks
      List<BrowsePath> assetPaths = computeComponentPaths(asset);
      appendPath(assetPaths, lastSegment(asset.path()), asset.path());
      return assetPaths;

    }).orElseGet(() -> super.computeAssetPaths(asset));
  }

  /**
   * Generates browse nodes to the component using the standard Maven layout.
   */
  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    checkNotNull(asset);

    Component component = asset.component().get(); // NOSONAR: caller guarantees this

    List<String> componentPath = pathToArtifactFolder(component);

    String version = component.version();
    if (!isBlank(version) && !version.equals(componentPath.get(componentPath.size() - 1))) {
      componentPath.add(version);
    }

    return fromPaths(componentPath, true);
  }

  /**
   * Generates a path to the artifact folder using the standard Composer layout.
   * <ul>
   * <li>Schema: /vendor/project/version/
   * </ul>
   */
  private List<String> pathToArtifactFolder(Component component) {
    List<String> paths = new ArrayList<>();

    String vendor = component.namespace();
    String project = component.name();
    String version = component.version();

    paths.add(vendor);
    paths.add(project);
    paths.add(version);

    return paths;
  }
}
