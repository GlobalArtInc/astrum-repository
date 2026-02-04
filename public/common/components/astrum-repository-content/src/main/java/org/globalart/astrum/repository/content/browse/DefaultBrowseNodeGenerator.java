
package org.globalart.astrum.repository.content.browse;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.browse.node.BrowsePathBuilder;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

import org.springframework.beans.factory.annotation.Qualifier;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Component-led layout based on group, name, and version; places assets one level below their components.
 *
 * @since 3.24
 */
@org.springframework.stereotype.Component
@Qualifier(DefaultBrowseNodeGenerator.NAME)
public class DefaultBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
  public static final String NAME = "default";

  /**
   * @return componentPath/assetName if asset has an component, otherwise assetPath
   */
  @Override
  public List<BrowsePath> computeAssetPaths(final Asset asset) {
    checkNotNull(asset);

    if (asset.component().isPresent()) {
      // place asset one level beneath component, using the asset's name as the display name
      List<BrowsePath> paths = computeComponentPaths(asset);
      String lastSegment = lastSegment(asset.path());
      BrowsePathBuilder.appendPath(paths, lastSegment);
      return paths;
    }
    else {
      return super.computeAssetPaths(asset);
    }
  }

  /**
   * @return [componentGroup/]componentName/[componentVersion/]
   */
  @Override
  public List<BrowsePath> computeComponentPaths(final Asset asset) {
    checkNotNull(asset);

    List<String> paths = new ArrayList<>();

    checkArgument(asset.component().isPresent());
    Component component = asset.component().get();
    if (!Strings2.isBlank(component.namespace())) {
      paths.add(component.namespace());
    }
    paths.add(component.name());
    if (!Strings2.isBlank(component.version())) {
      paths.add(component.version());
    }
    return BrowsePathBuilder.fromPaths(paths, true);
  }
}
