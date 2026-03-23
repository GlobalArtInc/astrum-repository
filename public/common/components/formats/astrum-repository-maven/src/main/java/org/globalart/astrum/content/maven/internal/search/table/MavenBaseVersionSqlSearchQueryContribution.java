
package org.globalart.astrum.content.maven.internal.search.table;

import java.util.Optional;

import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.SqlSearchQueryContributionSupport;
import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.StringTerm;

import static org.apache.commons.lang3.StringUtils.isNotBlank;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Creates sql query conditions for the assets.attributes.baseVersion search term.
 */
@Component
@Qualifier(MavenBaseVersionSqlSearchQueryContribution.NAME)
public class MavenBaseVersionSqlSearchQueryContribution
    extends SqlSearchQueryContributionSupport
{
  protected static final String BASE_VERSION = "attributes.maven2.baseVersion";

  public static final String NAME = BASE_VERSION;

  @Override
  public Optional<Expression> createPredicate(final SearchFilter searchFilter) {
    String value = searchFilter.getValue();
    if (isNotBlank(value)) {
      return super.createPredicate(new SearchFilter(searchFilter.getProperty(), value.trim()));
    }
    else {
      return super.createPredicate(searchFilter);
    }
  }

  @Override
  protected StringTerm createMatchTerm(final boolean exact, final String match) {
    return new ExactTerm(match);
  }
}
