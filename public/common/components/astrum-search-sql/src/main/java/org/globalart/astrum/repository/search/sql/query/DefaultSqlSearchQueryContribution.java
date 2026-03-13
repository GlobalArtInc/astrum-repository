
package org.globalart.astrum.repository.search.sql.query;

import org.globalart.astrum.repository.search.sql.SqlSearchQueryContribution;
import org.globalart.astrum.repository.search.sql.SqlSearchQueryContributionSupport;
import org.globalart.astrum.repository.search.sql.query.syntax.StringTerm;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Base implementation for {@link SqlSearchQueryContribution}
 *
 * @since 3.38
 */
@Component
@Qualifier("default")
public class DefaultSqlSearchQueryContribution
    extends SqlSearchQueryContributionSupport
{
  @Override
  protected StringTerm createMatchTerm(final boolean exact, final String value) {
    // Use parent implementation which respects the exact parameter
    // When exact=true, creates ExactTerm for exact matching using the non-tsvector column
    // When exact=false, creates LenientTerm for case-insensitive full-text search
    return super.createMatchTerm(exact, value);
  }
}
