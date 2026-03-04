
package org.globalart.astrum.selector;

import org.apache.commons.jexl3.parser.ASTJexlScript;

/**
 * Walks the {@link ASTJexlScript} script, transforming CSEL expressions and populating the {@link SelectorSqlBuilder}
 * builder.
 *
 * @since 3.24
 */
public interface CselToSql<T>
{
  /**
   * Transforms the given CSEL expression (in script form) to SQL for use in a 'where' clause.
   *
   * @param script the CSEL script to transform
   * @param builder the SQL builder to use
   */
  void transformCselToSql(final ASTJexlScript script, final T builder);
}
