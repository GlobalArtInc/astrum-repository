
package org.globalart.astrum.script;

/**
 * Emitted when a {@link Script} is deleted.
 *
 * @since 3.1
 */
public class ScriptDeletedEvent
    extends ScriptEvent
{
  public ScriptDeletedEvent(final Script script) {
    super(script);
  }
}
