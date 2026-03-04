
package org.globalart.astrum.selector;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Subset of JEXL selectors that can also be represented as SQL.
 *
 * @since 3.6
 */
public class CselSelector
    extends JexlSelector
{
  public static final String TYPE = "csel";

  private final CselToSql cselToSql;

  public CselSelector(final CselToSql cselToSql, final JexlExpression expression) {
    super(expression);
    this.cselToSql = checkNotNull(cselToSql);

  }

  @Override
  public void toSql(final SelectorSqlBuilder sqlBuilder) {
    cselToSql.transformCselToSql(expression.getSyntaxTree(), sqlBuilder);
  }

  @Override
  public <T> void toSql(final T sqlBuilder, final CselToSql<T> cselToSql) {
    cselToSql.transformCselToSql(expression.getSyntaxTree(), sqlBuilder);
  }
}
