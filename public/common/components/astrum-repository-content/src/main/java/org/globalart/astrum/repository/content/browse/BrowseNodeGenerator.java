
package org.globalart.astrum.repository.content.browse;

import java.util.List;

import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;

import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH_CHAR;

/**
 * Defines the browse node layout for assets and their components of the same format.
 *
 * @since 3.24
 */
public interface BrowseNodeGenerator
{
  /**
   * Computes the paths leading to the asset in the browse tree.
   */
  List<BrowsePath> computeAssetPaths(Asset asset);

  /**
   * Computes the paths leading to the asset's component in the browse tree.
   */
  List<BrowsePath> computeComponentPaths(Asset asset);

  /**
   * Whether this format supports multiple assets per component. Formats that have a 1:1 mapping can streamline some
   * queries
   */
  boolean hasMultipleAssetsPerComponent();

  /**
   * Finds the last segment of the given path.
   */
  default String lastSegment(final String path) {
    int lastNonSlash = path.length() - 1;
    while (lastNonSlash >= 0 && path.charAt(lastNonSlash) == SLASH_CHAR) {
      lastNonSlash--;
    }
    int precedingSlash = path.lastIndexOf(SLASH_CHAR, lastNonSlash - 1);
    return path.substring(precedingSlash + 1, lastNonSlash + 1);
  }
}
