
package org.globalart.astrum.script;

/**
 * {@link Script} event.
 *
 * @since 3.1
 */
public class ScriptEvent
{
  private final Script script;

  public ScriptEvent(final Script script) {
    this.script = script;
  }

  public Script getScript() {
    return script;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "script=" + script +
        '}';
  }
}
