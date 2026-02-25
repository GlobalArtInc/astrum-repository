
package org.globalart.astrum.repository.search.sql.query.syntax;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Iterables;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An SQL clause which uses the same operand for a collection of predicates, e.g. {@code foo = 'bar' AND bar = 'foo'}
 */
public class SqlClause
    implements Expression
{
  private final Operand operand;

  private final List<Expression> expressions;

  /*
   * private constructor to allow us to optimize in the create method
   */
  private SqlClause(final Operand operand, final List<? extends Expression> expressions) {
    this.operand = checkNotNull(operand);
    checkArgument(operand == Operand.AND || operand == Operand.OR, "Unexpected operand: " + operand.toString());
    checkArgument(expressions.size() > 1, "Must have at least 2 expressions");
    this.expressions = ImmutableList.copyOf(checkNotNull(expressions));
  }

  /**
   * The expressions in this clause which are conjoined by the operand
   */
  public List<Expression> expressions() {
    return Collections.unmodifiableList(expressions);
  }

  @Override
  public Operand operand() {
    return operand;
  }

  @Override
  public int hashCode() {
    return Objects.hash(operand, expressions);
  }

  @Override
  public boolean equals(final Object obj) {
    if (this == obj)
      return true;
    if (obj == null)
      return false;
    if (getClass() != obj.getClass())
      return false;
    SqlClause other = (SqlClause) obj;

    return operand == other.operand && Objects.equals(expressions, other.expressions);
  }

  @Override
  public String toString() {
    return expressions.stream()
        .map(Object::toString)
        .collect(Collectors.joining(' ' + operand.toString() + ' ', "(", ")"));
  }

  public static Expression create(final Operand operand, final Expression... expressions) {
    if (expressions.length == 1) {
      // optimization we avoid creating the clause and return the only expression provided
      return expressions[0];
    }
    return new SqlClause(operand, Arrays.asList(expressions));
  }

  public static Expression create(final Operand operand, final List<? extends Expression> expressions) {
    if (expressions.size() == 1) {
      // optimization we avoid creating the clause and return the only expression provided
      return Iterables.getOnlyElement(expressions);
    }
    return new SqlClause(operand, expressions);
  }
}
