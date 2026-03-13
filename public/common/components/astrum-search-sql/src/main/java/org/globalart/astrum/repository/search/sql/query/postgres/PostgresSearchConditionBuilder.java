package org.globalart.astrum.repository.search.sql.query.postgres;

import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

import org.globalart.astrum.repository.search.sql.query.SqlSearchQueryCondition;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.NullTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.RegexWildcardTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.SingleValueTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlClause;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;
import org.globalart.astrum.repository.search.sql.query.syntax.Term;
import org.globalart.astrum.repository.search.sql.query.syntax.TermCollection;
import org.globalart.astrum.rest.ValidationErrorsException;

import com.google.common.collect.Iterables;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.apache.commons.lang3.StringUtils.replace;

abstract class PostgresSearchConditionBuilder
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  protected static final String BRACKET_CLOSE = ")";

  protected static final String BRACKET_OPEN = "(";

  protected final PostgresSearchDB db;

  protected final Map<String, Object> parameters = new HashMap<>();

  protected final ValidationErrorsException errors = new ValidationErrorsException();

  private int parameterIndex = 0;

  private final ConditionType conditionType;

  public enum ConditionType
  {
    COMPONENT_FILTER("filterParams"),
    ASSET_FILTER("assetFilterParams");

    private final String parameterName;

    ConditionType(final String parameterName) {
      this.parameterName = parameterName;
    }

    public String getParameterName() {
      return parameterName;
    }
  }

  PostgresSearchConditionBuilder(final PostgresSearchDB db) {
    this(db, ConditionType.COMPONENT_FILTER);
  }

  PostgresSearchConditionBuilder(final PostgresSearchDB db, final ConditionType conditionType) {
    this.db = checkNotNull(db);
    this.conditionType = checkNotNull(conditionType);
  }

  public SqlSearchQueryCondition build(final Expression expression) {
    Optional<String> query = process(expression)
        .map(CharSequence::toString)
        .map(String::trim);

    if (!errors.getValidationErrors().isEmpty()) {
      throw errors;
    }

    return new SqlSearchQueryCondition(query.orElse(""), parameters);
  }

  private Optional<CharSequence> process(final Expression expression) {
    if (expression instanceof SqlClause clause) {
      return process(clause);
    }
    if (expression instanceof SqlPredicate predicate) {
      return process(predicate);
    }
    errors.withError("Unknown expression " + expression);
    return Optional.empty();
  }

  private Optional<CharSequence> process(final SqlClause clause) {
    if (clause.expressions().size() == 1) {
      return process(Iterables.getOnlyElement(clause.expressions()));
    }

    String operator = clause.operand() == Operand.AND ? " AND " : " OR ";

    List<CharSequence> parts = clause.expressions()
        .stream()
        .map(this::process)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .collect(Collectors.toList());

    if (parts.isEmpty()) {
      return Optional.empty();
    }

    return Optional.of(BRACKET_OPEN + String.join(operator, parts) + BRACKET_CLOSE);
  }

  private Optional<CharSequence> process(final SqlPredicate predicate) {
    Optional<PostgresSearchColumn> optColumn = db.getColumn(predicate.getSearchField());
    if (optColumn.isEmpty()) {
      log.error("Unknown column for search {}", predicate.getSearchField());
      return Optional.empty();
    }

    PostgresSearchColumn column = optColumn.get();
    Collection<SingleValueTerm<?>> terms = toList(predicate.getTerm());
    Operand op = predicate.operand();

    if (terms.isEmpty()) {
      return op == Operand.IN || op == Operand.ANY ? Optional.of("FALSE") : Optional.empty();
    }

    if (!op.supportsMultiple() && terms.size() != 1) {
      errors.withError(
          String.format("Unsupported number of terms for operand %s for %s with terms %s", op, column.describe(),
              terms));
      return Optional.empty();
    }

    if (isNullTerm(terms)) {
      return createNullCheck(column, op);
    }

    return createQuery(column, op, terms);
  }

  protected Optional<CharSequence> createQuery(
      final PostgresSearchColumn column,
      final Operand op,
      final Collection<SingleValueTerm<?>> terms)
  {
    return createExactPredicate(column, op, terms);
  }

  private Optional<CharSequence> createNullCheck(final PostgresSearchColumn column, final Operand op) {
    return switch (op) {
      case EQ -> Optional.of(column.nullCheckColumn() + " IS NULL");
      case NOT_EQ -> Optional.of(column.nullCheckColumn() + " IS NOT NULL");
      default -> {
        errors.withError("Unexpected operator " + op + " for null value on column " + column.describe());
        yield Optional.empty();
      }
    };
  }

  private Optional<CharSequence> createExactPredicate(
      final PostgresSearchColumn column,
      final Operand op,
      final Collection<SingleValueTerm<?>> terms)
  {
    if (op == Operand.REGEX) {
      return createRegexExpression(column, Iterables.getOnlyElement(terms));
    }

    if (column.columnName().isEmpty()) {
      errors.withError("Unexpected operator " + op + " for column " + column.describe());
      return Optional.empty();
    }

    String columnName = column.columnName().get();
    return switch (op) {
      case EQ -> createSimpleExpression(column, columnName, "=", Iterables.getOnlyElement(terms));
      case NOT_EQ -> createSimpleExpression(column, columnName, "<>", Iterables.getOnlyElement(terms));
      case IN, ANY -> createInExpression(column, columnName, terms);
      default -> {
        errors.withError("Unexpected operator " + op + " for column " + column.describe());
        yield Optional.empty();
      }
    };
  }

  private Optional<CharSequence> createSimpleExpression(
      final PostgresSearchColumn column,
      final String columnName,
      final String operator,
      final SingleValueTerm<?> term)
  {
    String param = param(columnName);
    parameters.put(param, term.get());
    return Optional.of(columnName + ' ' + operator + ' ' + column.cast(placeholder(param)));
  }

  private Optional<CharSequence> createRegexExpression(
      final PostgresSearchColumn column,
      final SingleValueTerm<?> term)
  {
    if (column.regexColumnName().isEmpty() || !column.isText()) {
      errors.withError("Regular expressions are not supported for column " + column.describe());
      return Optional.empty();
    }

    String columnName = column.regexColumnName().get();
    String param = param(columnName);
    Object value = term instanceof RegexWildcardTerm regexTerm ? regexTerm.toRegexPattern() : term.get();
    parameters.put(param, value);

    return Optional.of(columnName + " ~ " + placeholder(param));
  }

  private Optional<CharSequence> createInExpression(
      final PostgresSearchColumn column,
      final String columnName,
      final Collection<SingleValueTerm<?>> terms)
  {
    List<String> placeholders = new ArrayList<>();
    for (SingleValueTerm<?> term : terms) {
      String param = param(columnName);
      parameters.put(param, term.get());
      placeholders.add(column.cast(placeholder(param)));
    }

    return Optional.of(placeholders.stream()
        .collect(Collectors.joining(", ", columnName + " IN (", BRACKET_CLOSE)));
  }

  protected String param(final String columnName) {
    return replace(columnName, ".", "_") + parameterIndex++;
  }

  protected String placeholder(final String parameterName) {
    return "#{" + conditionType.getParameterName() + "." + parameterName + "}";
  }

  protected static Collection<SingleValueTerm<?>> toList(final Term term) {
    if (term instanceof TermCollection tc) {
      return tc.get();
    }
    return List.of((SingleValueTerm<?>) term);
  }

  private static boolean isNullTerm(final Collection<SingleValueTerm<?>> terms) {
    return terms.stream().allMatch(NullTerm.class::isInstance);
  }
}
