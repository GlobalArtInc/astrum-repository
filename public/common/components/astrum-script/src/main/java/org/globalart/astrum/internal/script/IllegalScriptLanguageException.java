
package org.globalart.astrum.internal.script;

/**
 * A scripting language that is not supported has been attempted.
 *
 * @since 3.22
 */
public class IllegalScriptLanguageException
    extends RuntimeException
{
  public IllegalScriptLanguageException(final String message) {
    super(message);
  }
}
