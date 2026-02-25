
package org.globalart.astrum.repository.search.sql.query.syntax;

import java.util.Objects;

import org.globalart.astrum.repository.rest.sql.SearchField;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A predicate used in an SQL query, e.g. {@code foo = 'bar'}
 */
public class SqlPredicate
    implements Expression
{
  private final Operand operand;

  private final SearchField searchField;

  private final Term term;

  public SqlPredicate(final Operand operand, final SearchField searchField, final Term term) {
    this.operand = checkNotNull(operand);
    this.searchField = checkNotNull(searchField);
    this.term = checkNotNull(term);
  }

  @Override
  public Operand operand() {
    return operand;
  }

  /**
   * The database field on the left of the operand in this predicate.
   */
  public SearchField getSearchField() {
    return searchField;
  }

  /**
   * The term on the right of the operand in this predicate.
   */
  public Term getTerm() {
    return term;
  }

  @Override
  public int hashCode() {
    return Objects.hash(operand, searchField, term);
  }

  @Override
  public boolean equals(final Object obj) {
    if (this == obj)
      return true;
    if (obj == null)
      return false;
    if (getClass() != obj.getClass())
      return false;
    SqlPredicate other = (SqlPredicate) obj;
    return operand == other.operand && searchField == other.searchField && Objects.equals(term, other.term);
  }

  @Override
  public String toString() {
    return "SqlPredicate [searchField=" + searchField + ", operand=" + operand + ", term=" + term + "]";
  }
}
