
package org.globalart.astrum.scheduling.events;

/**
 * Fired after recovery mode transitions from enabled to disabled.
 */
public final class RecoveryModeDisabledEvent
    extends RecoveryModeEvent
{
  public RecoveryModeDisabledEvent() {
    super(false);
  }
}
