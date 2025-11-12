
package org.globalart.astrum.internal.log.overrides.datastore;

import org.globalart.astrum.common.event.EventWithSource;

/**
 * An event fired when the logger overrides has changed in order to propagate changes to all nodes.
 */
public class LoggerOverridesEvent
    extends EventWithSource
{
  private String name;

  private String level;

  private Action action;

  public LoggerOverridesEvent() {
    // deserialization
  }

  public LoggerOverridesEvent(final String name, final String level, final Action action) {
    this.name = name;
    this.level = level;
    this.action = action;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getLevel() {
    return level;
  }

  public void setLevel(final String level) {
    this.level = level;
  }

  public Action getAction() {
    return action;
  }

  public void setAction(final Action action) {
    this.action = action;
  }

  public enum Action
  {
    CHANGE,
    RESET,
    RESET_ALL
  }
}
