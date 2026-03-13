
package org.globalart.astrum.repository.search.sql.query;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.SqlSearchQueryContributionSupport;
import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlClause;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;
import org.globalart.astrum.repository.search.sql.query.syntax.StringTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.WildcardTerm;

import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * A keyword search is one where the user does not specify a specific
 * search field, rather expects the term to be matched across a number of core fields.
 *
 * A search term of the form: "group:name[:version][:extension][:classifier]" results in an exact search condition for
 * maven components with the specified group, name, version and optional extension and classifier.
 *
 * Otherwise we check the keywords field in the search index
 *
 * @since 3.38
 */
@Component
@Qualifier(KeywordSqlSearchQueryContribution.NAME)
public class KeywordSqlSearchQueryContribution
    extends SqlSearchQueryContributionSupport
{
  public static final String NAME = "keyword";

  /**
   * Allow dependency searches of the form "group:name[:version][:extension][:classifier]"
   */
  private static final String GAVEC_REGEX =
      "^(?<group>[^\\s:]+):(?<name>[^\\s:]+)(:(?<version>[^\\s:]+))?(:(?<extension>[^\\s:]+))?(:(?<classifier>[^\\s:]+))?$";

  private static final Pattern GAVEC_SPLITTER = Pattern.compile(GAVEC_REGEX);

  @Override
  public Optional<Expression> createPredicate(final SearchFilter filter) {
    log.debug("Creating predicate for {}", filter);

    if (filter == null) {
      return Optional.empty();
    }

    final String value = filter.getValue();

    if (Strings2.isBlank(value)) {
      return Optional.empty();
    }

    Matcher gavSearchMatcher = GAVEC_SPLITTER.matcher(value.trim());

    if (gavSearchMatcher.matches()) {
      return createGavPredicate(gavSearchMatcher);
    }
    return super.createPredicate(filter);
  }

  private Optional<Expression> createGavPredicate(final Matcher gavSearchMatcher) {
    List<Expression> expressions = new ArrayList<>();

    from(gavSearchMatcher, "group", SearchField.NAMESPACE).ifPresent(expressions::add);
    from(gavSearchMatcher, "name", SearchField.NAME).ifPresent(expressions::add);
    from(gavSearchMatcher, "version", SearchField.VERSION).ifPresent(expressions::add);
    // Ideally we look these up rather than using hard coded values
    from(gavSearchMatcher, "extension", SearchField.FORMAT_FIELD_2).ifPresent(expressions::add);
    from(gavSearchMatcher, "classifier", SearchField.FORMAT_FIELD_3).ifPresent(expressions::add);

    if (expressions.isEmpty()) {
      log.debug("Found no GAVEC expressions in {}", gavSearchMatcher);
      return Optional.empty();
    }

    log.debug("Created expressions for GAVEC {}", expressions);

    return Optional.of(SqlClause.create(Operand.AND, expressions));
  }

  @Override
  protected StringTerm createMatchTerm(final boolean exact, final String value) {
    return exact ? new ExactTerm(value) : new WildcardTerm(value);
  }

  private static Optional<Expression> from(final Matcher matcher, final String name, final SearchField field) {
    String val = matcher.group(name);
    if (Strings2.isBlank(val)) {
      return Optional.empty();
    }
    return Optional.of(new SqlPredicate(Operand.EQ, field, new ExactTerm(val)));
  }
}
