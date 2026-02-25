
package org.globalart.astrum.repository.search.sql;

import java.util.Optional;

import org.globalart.astrum.repository.search.SearchRequest;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;

/**
 * Allows creation of sql search query condition(s) based on a given SearchFilter.
 *
 * @see SearchFilter
 * @see SqlSearchQueryCondition
 * @since 3.38
 */
public interface SqlSearchQueryContribution
{
  /**
   * Creates {@link Expresion}(s) for the specified SearchFilter
   */
  Optional<Expression> createPredicate(SearchFilter filter);

  default SearchRequest modifyRequest(SearchRequest request) {
    return request;
  }
}
