package org.globalart.astrum.repository.search.sql.query.postgres;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Collection;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.search.sql.index.SearchTokenizer;
import org.globalart.astrum.repository.search.sql.query.syntax.BooleanTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.LenientTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.RegexWildcardTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.SingleValueTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.StringTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.WildcardTerm;

import com.google.common.collect.Iterables;

import static org.globalart.astrum.repository.search.sql.index.SearchTokenizer.stripLeadingSeparators;
import static org.globalart.astrum.repository.search.sql.index.SearchTokenizer.stripTrailingSeparators;

public class PostgresFulltextSearchConditionBuilder
    extends PostgresSearchConditionBuilder
{
  static final int MAX_LEXEME_BYTES = 2046;

  private static final String PREFIX = ":*";

  private static final String FOLLOWED_BY = " <-> ";

  PostgresFulltextSearchConditionBuilder(final PostgresSearchDB db) {
    super(db);
  }

  PostgresFulltextSearchConditionBuilder(final PostgresSearchDB db, final ConditionType conditionType) {
    super(db, conditionType);
  }

  @Override
  protected Optional<CharSequence> createQuery(
      final PostgresSearchColumn column,
      final Operand op,
      final Collection<SingleValueTerm<?>> terms)
  {
    if (!column.isText()) {
      if (terms.stream().allMatch(term -> term instanceof ExactTerm || term instanceof BooleanTerm)) {
        return super.createQuery(column, op, terms);
      }
      errors.withError("Unsupported search term " + terms + " for column " + column.describe());
      return Optional.empty();
    }

    if (op == Operand.REGEX || isExactMatch(column, op, terms)) {
      return super.createQuery(column, op, terms);
    }

    if (terms.stream().allMatch(term -> term instanceof StringTerm st && Strings2.isBlank(st.get()))) {
      return Optional.empty();
    }

    if (column.hasColumn() && op != Operand.NOT_EQ && terms.size() == 1
        && Iterables.getOnlyElement(terms) instanceof WildcardTerm term) {
      return createPrefixExpression(column, term);
    }

    if (column.supportsTextSearch()) {
      return createTextSearch(column, op, terms);
    }

    return createContainsExpression(column, op, terms);
  }

  private static boolean isExactMatch(
      final PostgresSearchColumn column,
      final Operand op,
      final Collection<SingleValueTerm<?>> terms)
  {
    if (!column.hasColumn()) {
      return false;
    }
    if (op == Operand.NOT_EQ) {
      return true;
    }
    if (op == Operand.IN || op == Operand.ANY) {
      return terms.stream().allMatch(ExactTerm.class::isInstance);
    }
    if (terms.size() != 1) {
      return false;
    }
    SingleValueTerm<?> term = Iterables.getOnlyElement(terms);
    return term instanceof ExactTerm || term instanceof BooleanTerm || term instanceof RegexWildcardTerm;
  }

  private Optional<CharSequence> createTextSearch(
      final PostgresSearchColumn column,
      final Operand op,
      final Collection<SingleValueTerm<?>> terms)
  {
    String operator;
    switch (op) {
      case EQ:
      case NOT_EQ:
        operator = " & ";
        break;
      case ANY:
      case IN:
        operator = " | ";
        break;
      default:
        errors.withError("Unsupported operand " + op + " for terms " + terms);
        return Optional.empty();
    }

    List<String> queries = terms.stream()
        .map(this::toTsQuery)
        .flatMap(Optional::stream)
        .collect(Collectors.toList());

    if (queries.isEmpty()) {
      return Optional.empty();
    }

    String tsquery = queries.size() == 1
        ? queries.get(0)
        : queries.stream().map(query -> BRACKET_OPEN + query + BRACKET_CLOSE).collect(Collectors.joining(operator));

    String tsvectorColumn = column.tsvectorColumnName().orElseThrow();
    String param = param(tsvectorColumn);
    parameters.put(param, tsquery);

    String predicate = tsvectorColumn + " @@ " + placeholder(param) + "::tsquery";
    if (op == Operand.NOT_EQ) {
      return Optional.of("NOT (" + predicate + ")");
    }
    return Optional.of(predicate);
  }

  private Optional<String> toTsQuery(final SingleValueTerm<?> term) {
    if (term instanceof WildcardTerm wildcard) {
      return query(removeTrailingWildcard(wildcard.get()), wildcard.isAllowTokenization(), true);
    }
    if (term instanceof LenientTerm lenient) {
      return query(lenient.get(), true, false);
    }
    return query(String.valueOf(term.get()), false, false);
  }

  private Optional<String> query(final String value, final boolean tokenize, final boolean prefix) {
    if (Strings2.isBlank(value)) {
      return Optional.empty();
    }
    if (!tokenize) {
      return Optional.of(lexeme(value, prefix));
    }

    String stripped = stripTrailingSeparators(stripLeadingSeparators(value));
    if (Strings2.isBlank(stripped)) {
      return Optional.empty();
    }

    Set<String> alternatives = new LinkedHashSet<>();
    alternatives.add(lexeme(value, prefix));
    alternatives.add(lexeme(stripped, prefix));

    List<String> tokens = Arrays.stream(SearchTokenizer.TOKENIZER.split(value))
        .filter(Strings2::notBlank)
        .collect(Collectors.toList());

    if (!tokens.isEmpty()) {
      alternatives.add(IntStream.range(0, tokens.size())
          .mapToObj(i -> lexeme(tokens.get(i), prefix && i == tokens.size() - 1))
          .collect(Collectors.joining(FOLLOWED_BY)));
    }

    return Optional.of(String.join(" | ", alternatives));
  }

  private String lexeme(final String value, final boolean prefix) {
    if (value.getBytes(StandardCharsets.UTF_8).length > MAX_LEXEME_BYTES) {
      errors.withError("Search term is too long: " + value.substring(0, 64) + "...");
    }
    return SearchTokenizer.tsEscape(value) + (prefix ? PREFIX : "");
  }

  private Optional<CharSequence> createPrefixExpression(final PostgresSearchColumn column, final WildcardTerm term) {
    String value = removeTrailingWildcard(term.get());
    if (value.isBlank()) {
      return Optional.empty();
    }

    String columnName = column.columnName().orElseThrow();
    String likeParam = param(columnName);
    parameters.put(likeParam, escapeLike(value) + "%");
    String like = "LOWER(" + columnName + ") LIKE LOWER(" + placeholder(likeParam) + ")";

    if (!column.supportsTextSearch()) {
      return Optional.of(like);
    }

    String tsvectorColumn = column.tsvectorColumnName().get();
    String tsParam = param(tsvectorColumn);
    parameters.put(tsParam, lexeme(value, true));

    return Optional.of(BRACKET_OPEN + tsvectorColumn + " @@ " + placeholder(tsParam) + "::tsquery AND " + like
        + BRACKET_CLOSE);
  }

  private Optional<CharSequence> createContainsExpression(
      final PostgresSearchColumn column,
      final Operand op,
      final Collection<SingleValueTerm<?>> terms)
  {
    String operator;
    switch (op) {
      case EQ:
        operator = " AND ";
        break;
      case ANY:
      case IN:
        operator = " OR ";
        break;
      default:
        errors.withError("Unsupported operand " + op + " for terms " + terms);
        return Optional.empty();
    }

    String columnName = column.columnName().orElseThrow();
    List<String> expressions = terms.stream()
        .filter(term -> Strings2.notBlank(String.valueOf(term.get())))
        .map(term -> {
          String param = param(columnName);
          parameters.put(param, "%" + likeValue(term) + "%");
          return "LOWER(" + columnName + ") LIKE LOWER(" + placeholder(param) + ")";
        })
        .collect(Collectors.toList());

    if (expressions.isEmpty()) {
      return Optional.empty();
    }

    return Optional.of(BRACKET_OPEN + String.join(operator, expressions) + BRACKET_CLOSE);
  }

  private static String likeValue(final SingleValueTerm<?> term) {
    String value = escapeLike(String.valueOf(term.get()));
    if (term instanceof WildcardTerm) {
      return value.replace("*", "%");
    }
    return value;
  }

  private static String removeTrailingWildcard(final String value) {
    return value.replaceFirst("\\*$", "");
  }

  private static String escapeLike(final String value) {
    return value
        .replace("\\", "\\\\")
        .replace("%", "\\%")
        .replace("_", "\\_");
  }
}
