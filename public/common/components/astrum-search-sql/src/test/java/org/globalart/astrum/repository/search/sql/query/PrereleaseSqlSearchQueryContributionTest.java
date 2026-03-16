
package org.globalart.astrum.repository.search.sql.query;

import java.util.Optional;

import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.query.SearchFilter;
import org.globalart.astrum.repository.search.sql.query.syntax.BooleanTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class PrereleaseSqlSearchQueryContributionTest
{
  private PrereleaseSqlSearchQueryContribution underTest = new PrereleaseSqlSearchQueryContribution();

  @Test
  void shouldReturnEmptyForNullFilter() {
    Optional<Expression> result = underTest.createPredicate(null);

    assertFalse(result.isPresent());
  }

  @Test
  void shouldCreatePredicateForTrueValue() {
    Optional<Expression> result = underTest.createPredicate(
        new SearchFilter(PrereleaseSqlSearchQueryContribution.NAME, "true"));

    assertTrue(result.isPresent());
    assertThat(result.get(), is(new SqlPredicate(Operand.EQ, SearchField.PRERELEASE, new BooleanTerm(true))));
  }

  @Test
  void shouldCreatePredicateForFalseValue() {
    Optional<Expression> result = underTest.createPredicate(
        new SearchFilter(PrereleaseSqlSearchQueryContribution.NAME, "false"));

    assertTrue(result.isPresent());
    assertThat(result.get(), is(new SqlPredicate(Operand.EQ, SearchField.PRERELEASE, new BooleanTerm(false))));
  }

  @Test
  void shouldBeCaseInsensitiveForTrue() {
    Optional<Expression> result = underTest.createPredicate(
        new SearchFilter(PrereleaseSqlSearchQueryContribution.NAME, "True"));

    assertTrue(result.isPresent());
    assertThat(result.get(), is(new SqlPredicate(Operand.EQ, SearchField.PRERELEASE, new BooleanTerm(true))));
  }

  @Test
  void shouldBeCaseInsensitiveForFalse() {
    Optional<Expression> result = underTest.createPredicate(
        new SearchFilter(PrereleaseSqlSearchQueryContribution.NAME, "FALSE"));

    assertTrue(result.isPresent());
    assertThat(result.get(), is(new SqlPredicate(Operand.EQ, SearchField.PRERELEASE, new BooleanTerm(false))));
  }

  @Test
  void shouldHandleWhitespaceInValue() {
    Optional<Expression> result = underTest.createPredicate(
        new SearchFilter(PrereleaseSqlSearchQueryContribution.NAME, "  true  "));

    assertTrue(result.isPresent());
    assertThat(result.get(), is(new SqlPredicate(Operand.EQ, SearchField.PRERELEASE, new BooleanTerm(true))));
  }

  @Test
  void shouldThrowValidationErrorForInvalidValue() {
    SearchFilter filter = new SearchFilter(PrereleaseSqlSearchQueryContribution.NAME, "invalid");

    ValidationErrorsException exception = assertThrows(ValidationErrorsException.class,
        () -> underTest.createPredicate(filter));

    assertThat(exception.getMessage(), is("Pre-release only supports true or false"));
  }

  @Test
  void shouldThrowValidationErrorForNumericValue() {
    SearchFilter filter = new SearchFilter(PrereleaseSqlSearchQueryContribution.NAME, "1");

    ValidationErrorsException exception = assertThrows(ValidationErrorsException.class,
        () -> underTest.createPredicate(filter));

    assertThat(exception.getMessage(), is("Pre-release only supports true or false"));
  }

  @Test
  void shouldThrowValidationErrorForEmptyString() {
    SearchFilter filter = new SearchFilter(PrereleaseSqlSearchQueryContribution.NAME, "");

    ValidationErrorsException exception = assertThrows(ValidationErrorsException.class,
        () -> underTest.createPredicate(filter));

    assertThat(exception.getMessage(), is("Pre-release only supports true or false"));
  }
}
