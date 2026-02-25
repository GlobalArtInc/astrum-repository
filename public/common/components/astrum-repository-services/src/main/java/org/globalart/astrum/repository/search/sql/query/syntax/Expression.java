
package org.globalart.astrum.repository.search.sql.query.syntax;

/**
 * Represents a truthy expression for use in SQL such as {@code WHERE expression}.
 */
public interface Expression
{
  /**
   * The operand used by the expression
   */
  Operand operand();
}
