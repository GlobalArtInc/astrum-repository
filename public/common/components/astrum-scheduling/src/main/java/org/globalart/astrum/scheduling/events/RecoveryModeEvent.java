
package org.globalart.astrum.scheduling.events;

/**
 * Base event for recovery-mode state transitions.
 */
public abstract class RecoveryModeEvent
{
  private final boolean enabled;

  protected RecoveryModeEvent(final boolean enabled) {
    this.enabled = enabled;
  }

  public boolean isEnabled() {
    return enabled;
  }
}
