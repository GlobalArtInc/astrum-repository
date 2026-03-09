
package org.globalart.astrum.script;

/**
 * Emitted when a {@link Script} is run.
 *
 * @since 3.22
 */
public class ScriptRunEvent
    extends ScriptEvent
{
  public ScriptRunEvent(final Script script) {
    super(script);
  }
}
