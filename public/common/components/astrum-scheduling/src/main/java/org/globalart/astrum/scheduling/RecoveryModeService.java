
package org.globalart.astrum.scheduling;

public interface RecoveryModeService
{
  /**
   * @return true when recovery mode is enabled, false otherwise
   */
  boolean isRecoveryMode();

  /**
   * Enables recovery mode and cancels conflicting running tasks.
   */
  void enableRecoveryMode();

  /**
   * Disables recovery mode
   */
  void disableRecoveryMode();

  /**
   * Ensures that recovery mode is not active, throwing an IllegalStateException if it is.
   *
   * @param taskName the name of the task being executed
   * @throws IllegalStateException if recovery mode is active
   */
  void ensureNotInRecoveryMode(String taskName);
}
