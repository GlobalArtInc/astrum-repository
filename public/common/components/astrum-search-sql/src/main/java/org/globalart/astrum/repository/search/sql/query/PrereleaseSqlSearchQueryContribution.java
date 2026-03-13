
package org.globalart.astrum.repository.search.sql.query;

import java.util.Optional;

import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.SqlSearchQueryContribution;
import org.globalart.astrum.repository.search.sql.query.syntax.BooleanTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.repository.search.sql.query.syntax.Operand.EQ;

@Component
@Qualifier(PrereleaseSqlSearchQueryContribution.NAME)
public class PrereleaseSqlSearchQueryContribution
    implements SqlSearchQueryContribution
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String NAME = "isPrerelease";

  @Override
  public Optional<Expression> createPredicate(SearchFilter filter) {
    log.debug("Creating predicate for {}", filter);

    if (filter == null) {
      return Optional.empty();
    }

    String valueString = filter.getValue().trim();
    Boolean value = Boolean.valueOf(valueString);

    if (!value.toString().equalsIgnoreCase(valueString)) {
      throw new ValidationErrorsException("Pre-release only supports true or false");
    }

    return Optional.of(new SqlPredicate(EQ, SearchField.PRERELEASE, new BooleanTerm(value)));
  }
}
