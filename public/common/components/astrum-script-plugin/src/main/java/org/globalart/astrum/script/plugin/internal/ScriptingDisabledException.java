
package org.globalart.astrum.script.plugin.internal;

/**
 * An action on a script has been attempted when scripting is disabled.
 *
 * @since 3.22
 */
public class ScriptingDisabledException
    extends RuntimeException
{
  public ScriptingDisabledException(final String message) {
    super(message);
  }
}
