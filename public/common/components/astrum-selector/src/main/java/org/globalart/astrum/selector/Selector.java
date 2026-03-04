
package org.globalart.astrum.selector;

/**
 * An interface for evaluating selection based on variable input.
 *
 * @since 3.0
 */
public interface Selector
{
  /**
   * Returns {@code true} if this selector matches against the given variables, otherwise {@code false}.
   *
   * @param variableSource the source of variable values
   */
  boolean evaluate(VariableSource variableSource);

  /**
   * Returns SQL representing this selector for use as a 'where' clause against some queryable values.
   *
   * @param sqlBuilder the builder of 'where' clauses for content selectors
   *
   * @throws UnsupportedOperationException if this selector cannot be represented as SQL
   *
   * @since 3.16
   */
  void toSql(SelectorSqlBuilder sqlBuilder);

  /**
   * Returns SQL representing this selector for use as a 'where' clause against some queryable values.
   *
   * @param sqlBuilder used to accumulate 'where' clauses for content selectors
   * @param cselToSql transforms selector expressions into sql
   */
  <T> void toSql(T sqlBuilder, final CselToSql<T> cselToSql);
}
