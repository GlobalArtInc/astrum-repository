
package org.globalart.astrum.repository.search.sql.query;

import java.util.Optional;

import org.globalart.astrum.repository.rest.sql.SearchField;

/**
 * Abstraction for details specific to the underlying database used for search.
 */
public interface SearchDatabase
{
  String COMPONENT_PREFIX_ID = "cs.";

  String ASSET_PREFIX_ID = "ap.";

  String ATTRIBUTES = COMPONENT_PREFIX_ID + "attributes";

  /**
   * Return the column to use for sorting by the user specified {@link SearchField} if one exists.
   */
  Optional<String> getSortColumn(final SearchField field);

  /**
   * Format a JSON path expression for sorting on nested JSON attributes.
   * Different databases may have different functions or operators to build the query.
   *
   * @param columnName the column containing JSON (e.g., "cs.attributes")
   * @param sortAlias the sort alias which represents the JSON path inside the column (e.g., "maven2.extension")
   * @return database-specific JSON path expression for use in ORDER BY clauses
   */
  String formatJsonPath(String columnName, String sortAlias);
}
