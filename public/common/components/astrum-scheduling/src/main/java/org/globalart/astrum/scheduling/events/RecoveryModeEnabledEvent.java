
package org.globalart.astrum.scheduling.events;

/**
 * Fired after recovery mode transitions from disabled to enabled.
 */
public final class RecoveryModeEnabledEvent
    extends RecoveryModeEvent
{
  public RecoveryModeEnabledEvent() {
    super(true);
  }
}
