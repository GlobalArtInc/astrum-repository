
package org.globalart.astrum.repository.search.sql.query;

import java.util.Arrays;
import java.util.Optional;

import org.globalart.astrum.repository.rest.internal.DefaultSearchMappings;
import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.SearchMappingService;
import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlClause;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;
import org.globalart.astrum.repository.search.sql.query.syntax.StringTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.WildcardTerm;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_2;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_3;
import static org.globalart.astrum.repository.rest.sql.SearchField.KEYWORDS;
import static org.globalart.astrum.repository.rest.sql.SearchField.NAME;
import static org.globalart.astrum.repository.rest.sql.SearchField.NAMESPACE;
import static org.globalart.astrum.repository.rest.sql.SearchField.VERSION;

public class KeywordSqlSearchQueryContributionTest
{
  public static final String GAVEC_CONDITION_FORMAT = "GAVEC_CONDITION";

  private KeywordSqlSearchQueryContribution underTest;

  @Before
  public void setup() {
    underTest = new KeywordSqlSearchQueryContribution();
    underTest.init(new SearchMappingService(Arrays.asList(new DefaultSearchMappings())), false);
  }

  @Test
  public void shouldIgnoreNull() {
    Optional<Expression> result = underTest.createPredicate(null);

    assertTrue(!result.isPresent());
  }

  @Test
  public void shouldBeMavenGavecSearchCondition() {
    Optional<Expression> result =
        underTest.createPredicate(new SearchFilter("keyword", "org.mockito:mockito-core:3.24:jar:tests"));

    assertTrue(result.isPresent());

    assertThat(result.get(), is(SqlClause.create(Operand.AND, predicate(Operand.EQ, NAMESPACE, exact("org.mockito")),
        predicate(Operand.EQ, NAME, exact("mockito-core")), predicate(Operand.EQ, VERSION, exact("3.24")),
        predicate(Operand.EQ, FORMAT_FIELD_2, exact("jar")), predicate(Operand.EQ, FORMAT_FIELD_3, exact("tests")))));
  }

  @Test
  public void splitByAndSearch() {
    Optional<Expression> result = underTest.createPredicate(new SearchFilter("keyword", "mockito junit"));

    assertThat(result.get(),
        is(SqlClause.create(Operand.OR, new SqlPredicate(Operand.EQ, KEYWORDS, new WildcardTerm("mockito")),
            new SqlPredicate(Operand.EQ, KEYWORDS, new WildcardTerm("junit")))));
  }

  @Test
  public void shouldCreateExactTermWithDoubleQuotes() {
    Optional<Expression> result = underTest.createPredicate(new SearchFilter("keyword", "\"aether-util\""));

    assertTrue(result.isPresent());
    assertThat(result.get(), is(new SqlPredicate(Operand.EQ, KEYWORDS, new ExactTerm("aether-util"))));
  }

  @Test
  public void shouldCreateWildcardTermWithoutQuotes() {
    Optional<Expression> result = underTest.createPredicate(new SearchFilter("keyword", "aether-util"));

    assertTrue(result.isPresent());
    assertThat(result.get(), is(new SqlPredicate(Operand.EQ, KEYWORDS, new WildcardTerm("aether-util"))));
  }

  @Test
  public void shouldHandleMultipleQuotedTerms() {
    Optional<Expression> result = underTest.createPredicate(new SearchFilter("keyword", "\"aether\" \"util\""));

    assertTrue(result.isPresent());
    // Note: Empty string between quoted terms is also captured as ExactTerm
    assertThat(result.get(),
        is(SqlClause.create(Operand.OR, new SqlPredicate(Operand.EQ, KEYWORDS, new ExactTerm("aether")),
            new SqlPredicate(Operand.EQ, KEYWORDS, new ExactTerm("")),
            new SqlPredicate(Operand.EQ, KEYWORDS, new ExactTerm("util")))));
  }

  @Test
  public void shouldHandleMixedQuotedAndUnquoted() {
    Optional<Expression> result = underTest.createPredicate(new SearchFilter("keyword", "aether \"util\""));

    assertTrue(result.isPresent());
    assertThat(result.get(),
        is(SqlClause.create(Operand.OR, new SqlPredicate(Operand.EQ, KEYWORDS, new WildcardTerm("aether")),
            new SqlPredicate(Operand.EQ, KEYWORDS, new ExactTerm("util")))));
  }

  private static ExactTerm exact(final String value) {
    return new ExactTerm(value);
  }

  private static SqlPredicate predicate(final Operand operand, final SearchField searchField, final StringTerm term) {
    return new SqlPredicate(operand, searchField, term);
  }
}
