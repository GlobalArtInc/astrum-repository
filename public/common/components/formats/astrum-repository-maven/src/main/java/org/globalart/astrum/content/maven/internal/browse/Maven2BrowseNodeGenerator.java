
package org.globalart.astrum.content.maven.internal.browse;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;
import org.globalart.astrum.repository.maven.internal.Maven2Format;

import com.google.common.base.Splitter;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.text.Strings2.isBlank;
import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.appendPath;
import static org.globalart.astrum.repository.browse.node.BrowsePathBuilder.fromPaths;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Browse node generator for Maven.
 *
 * Maven layout is based on group, name, and version; places assets one level below their components.
 * This differs from the default generator in that any dots in the group are converted to slashes.
 *
 * Note: snapshot components are combined under their base version but keep their timestamped version:
 *
 * /org/globalart/astrum/astrum-common/3.7.0-SNAPSHOT/3.7.0-20171212.235354-266/
 *
 * This avoids having multiple snapshot components at the same '3.7.0-SNAPSHOT' node in the browse UI.
 *
 * A side-effect of this is snapshot assets are also displayed under the additional timestamped folder:
 *
 * /org/globalart/astrum/astrum-common/3.7.0-SNAPSHOT/3.7.0-20171107.223311-149/astrum-common-3.7.0-20171107.223311-149.pom
 * /org/globalart/astrum/astrum-common/3.7.0-SNAPSHOT/3.7.0-20171113.234015-168/astrum-common-3.7.0-20171113.234015-168.pom
 * /org/globalart/astrum/astrum-common/3.7.0-SNAPSHOT/3.7.0-20171212.235354-266/astrum-common-3.7.0-20171212.235354-266.pom
 *
 * instead being displayed under the same base '-SNAPSHOT' folder as in the standard Maven layout:
 *
 * /org/globalart/astrum/astrum-common/3.7.0-SNAPSHOT/astrum-common-3.7.0-20171107.223311-149.pom
 * /org/globalart/astrum/astrum-common/3.7.0-SNAPSHOT/astrum-common-3.7.0-20171113.234015-168.pom
 * /org/globalart/astrum/astrum-common/3.7.0-SNAPSHOT/astrum-common-3.7.0-20171212.235354-266.pom
 *
 * But note they still retain their real asset path as the 'request' path for permissions checks.
 *
 * @since 3.26
 */
@org.springframework.stereotype.Component
@Qualifier(Maven2Format.NAME)
public class Maven2BrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
  static final String BASE_VERSION = "baseVersion";

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

    // Components representing different timed snapshots would normally share the same artifact folder
    // in the standard Maven layout, for example 1.0-SNAPSHOT. But we want components to have a unique
    // entry in the browse UI so we give them each a timestamped folder under the base version folder.

    // So a component path of .../1.0-SNAPSHOT/ becomes .../1.0-SNAPSHOT/1.0-20171107.223311-1/

    String version = component.version();
    if (!isBlank(version) && !version.equals(componentPath.get(componentPath.size() - 1))) {
      componentPath.add(version);
    }

    return fromPaths(componentPath, true);
  }

  /**
   * Generates a path to the artifact folder using the standard Maven layout.
   * <ul>
   * <li>For snapshots: /org/some/group/myartifact/1.0-SNAPSHOT/
   * <li>For releases: /org/some/group/myartifact/1.0/
   * </ul>
   */
  private List<String> pathToArtifactFolder(Component component) {
    List<String> paths = new ArrayList<>();

    String groupId = component.namespace();
    String artifactId = component.name();
    String version = component.version();

    if (!isBlank(groupId)) {
      // convert dotted groupId to get the leading paths
      paths.addAll(Splitter.on('.').omitEmptyStrings().splitToList(groupId));
    }

    paths.add(artifactId);

    if (!isBlank(version)) {
      // versioned artifacts are all located under the same base version folder
      paths.add(baseVersion(component, version));
    }

    return paths;
  }

  /**
   * Attempts to get the 'base' Maven version if it has one, otherwise use the exact version.
   * <ul>
   * <li>Snapshots like 1.0-20171107.223311-1 have a base version of 1.0-SNAPSHOT
   * <li>Releases have base versions that exactly match their release version
   * </ul>
   */
  private String baseVersion(final Component component, final String version) {
    return component.attributes().child(Maven2Format.NAME).get(BASE_VERSION, String.class, version);
  }
}
