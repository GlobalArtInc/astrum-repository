
package org.globalart.astrum.repository.search.index;

import org.globalart.astrum.repository.Repository;

/**
 * Defines logic for determining if a repository search index is out of date and needs to be re-indexed.
 *
 * @since 3.37
 */
public interface SearchUpdateService
{
  String SEARCH_INDEX_OUTDATED = "search_index_outdated";

  boolean needsReindex(Repository repository);

  void doneReindexing(Repository repository);
}
