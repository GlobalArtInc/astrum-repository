
package org.globalart.astrum.repository.content.search.sql;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.search.sql.SearchRecord;

/**
 * Allows contribution of data from assets to {@link SearchRecord}
 *
 * May be used by formats to contribute custom fields to search indexes.
 */
public interface SearchCustomFieldContributor
{
  /**
   * From the given {@link Asset} the passed in {@link SearchRecord} will have its custom search fields populated.
   */
  void populateSearchCustomFields(SearchRecord searchRecord, Asset asset);

  /**
   * Should be an asset path searchable or not. By default, the path will be indexed.
   *
   * @param path the asset path.
   * @return {@code true} the path will be indexed, {@code false} otherwise.
   */
  default boolean isEnableSearchByPath(final String path) {
    return true;
  }
}
