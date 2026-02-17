
package org.globalart.astrum.repository.browse.node;

import org.springframework.beans.factory.annotation.Autowired;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Configuration options for browse tree
 *
 * @since 3.6
 */
@Component
public class BrowseNodeConfiguration
{
  private final boolean automaticRebuild;

  private final int rebuildPageSize;

  private final int deletePageSize;

  private final int maxNodes;

  private final int maxHtmlNodes;

  @Autowired
  public BrowseNodeConfiguration(
      @Value("${astrum.browse.component.tree.automaticRebuild:true}") final boolean automaticRebuild,
      @Value("${astrum.browse.component.tree.rebuildPageSize:1000}") final int rebuildPageSize,
      @Value("${astrum.browse.component.tree.deletePageSize:1000}") final int deletePageSize,
      @Value("${astrum.browse.component.tree.maxNodes:10000}") final int maxNodes,
      @Value("${astrum.browse.component.tree.maxHtmlNodes:10000}") final int maxHtmlNodes)
  {
    this.automaticRebuild = automaticRebuild;
    this.rebuildPageSize = rebuildPageSize;
    this.deletePageSize = deletePageSize;
    this.maxNodes = maxNodes;
    this.maxHtmlNodes = maxHtmlNodes;
  }

  @VisibleForTesting
  public BrowseNodeConfiguration() {
    this(true, 1000, 1000, 10_000, 10_000);
  }

  /**
   * The number of assets to retrieve at a time while rebuilding the browse tree
   */
  public int getRebuildPageSize() {
    return rebuildPageSize;
  }

  /**
   * The number of nodes to delete at a time while truncating the browse tree
   *
   * @since 3.7
   */
  public int getDeletePageSize() {
    return deletePageSize;
  }

  /**
   * The maximum number of nodes to display on a given level of a tree
   */
  public int getMaxNodes() {
    return maxNodes;
  }

  /**
   * The maximum number of nodes to display in the browse html view (for a given level of the tree)
   *
   * @since 3.6.1
   */
  public int getMaxHtmlNodes() {
    return maxHtmlNodes;
  }

  /**
   * Whether the tree should be automatically rebuilt if the number of assets is different from the number of leaves in
   * the browse tree on startup
   */
  public boolean isAutomaticRebuildEnabled() {
    return automaticRebuild;
  }
}
