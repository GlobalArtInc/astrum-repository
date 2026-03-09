
package org.globalart.astrum.script;

/**
 * Emitted when a {@link Script} is updated.
 *
 * @since 3.1
 */
public class ScriptUpdatedEvent
    extends ScriptEvent
{
  public ScriptUpdatedEvent(final Script script) {
    super(script);
  }
}
