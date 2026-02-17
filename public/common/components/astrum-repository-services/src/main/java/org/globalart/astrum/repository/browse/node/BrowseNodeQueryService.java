
package org.globalart.astrum.repository.browse.node;

import java.util.List;

import org.globalart.astrum.repository.Repository;

/**
 * Store providing access to the browse tree.
 *
 * @since 3.7
 */
public interface BrowseNodeQueryService
{
  /**
   * Returns the {@link BrowseNode}s directly visible under the given path.
   */
  Iterable<BrowseNode> getByPath(Repository repository, List<String> path, int maxNodes);

  /**
   * Converts {@link BrowseNode}s into {@link BrowseListItem}s for HTML browsing.
   */
  List<BrowseListItem> toListItems(Repository repository, Iterable<BrowseNode> nodes);
}
