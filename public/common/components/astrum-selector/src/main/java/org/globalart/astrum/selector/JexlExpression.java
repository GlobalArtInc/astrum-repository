
package org.globalart.astrum.selector;

import org.apache.commons.jexl3.internal.Engine;
import org.apache.commons.jexl3.internal.Script;
import org.apache.commons.jexl3.parser.ASTJexlScript;

/**
 * JEXL expression script that provides access to the underlying syntax tree.
 *
 * @since 3.16
 */
class JexlExpression
    extends Script
{
  public JexlExpression(final Engine engine, final String source, final ASTJexlScript script) {
    super(engine, source, script);
  }

  public ASTJexlScript getSyntaxTree() {
    return script;
  }
}
