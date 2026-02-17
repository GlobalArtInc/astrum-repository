
package org.globalart.astrum.repository.browse.node;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.app.VersionComparator;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import org.springframework.context.annotation.Primary;

/**
 * Sort using VersionComparator when dealing with two components, fall back to node name when dealing
 * with any other comparison of the same type (component/asset/folder), finally fall back to node type
 *
 * @since 3.13
 */
@Component
@Qualifier(DefaultBrowseNodeComparator.NAME)
@Primary
public class DefaultBrowseNodeComparator
    implements BrowseNodeComparator
{
  public static final String NAME = "DefaultBrowseNodeComparator";

  private final VersionComparator versionComparator;

  public static final int NODE_PRIORITY_COMPONENT = 1;

  public static final int NODE_PRIORITY_FOLDER = 2;

  public static final int NODE_PRIORITY_ASSET = 3;

  @Autowired
  public DefaultBrowseNodeComparator(final VersionComparator versionComparator) {
    this.versionComparator = versionComparator;
  }

  @Override
  public int compare(final BrowseNode o1, final BrowseNode o2) {
    int o1Priority = getNodePriorityLevel(o1);
    int o2Priority = getNodePriorityLevel(o2);

    if (o1Priority == NODE_PRIORITY_COMPONENT && o2Priority == NODE_PRIORITY_COMPONENT) {
      try {
        return versionComparator.compare(o1.getName(), o2.getName());
      }
      catch (IllegalArgumentException e) { // NOSONAR
        return 0;
      }
    }

    if (o1Priority == o2Priority) {
      return o1.getName().compareToIgnoreCase(o2.getName());
    }

    return Integer.compare(o1Priority, o2Priority);
  }

  /**
   * Returns an integer representing the priority level of the BrowseNode for comparison.
   * Priority is determined by the presence of component or asset identifiers, with folder nodes
   * having the highest priority.
   *
   * @param browseNode browseNode the node to evaluate for priority
   * @return an integer indicating the BrowseNode's priority level. A component node has the lowest priority,
   *         followed by asset nodes, then folder nodes.
   */
  public static int getNodePriorityLevel(final BrowseNode browseNode) {
    if (browseNode.getComponentId() != null) {
      return NODE_PRIORITY_COMPONENT;
    }

    if (browseNode.getAssetId() != null) {
      return NODE_PRIORITY_ASSET;
    }

    return NODE_PRIORITY_FOLDER;
  }
}
