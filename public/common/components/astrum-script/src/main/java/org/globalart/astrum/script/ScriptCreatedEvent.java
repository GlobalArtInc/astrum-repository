
package org.globalart.astrum.script;

/**
 * Emitted when a {@link Script} is created.
 *
 * @since 3.1
 */
public class ScriptCreatedEvent
    extends ScriptEvent
{
  public ScriptCreatedEvent(final Script script) {
    super(script);
  }
}
