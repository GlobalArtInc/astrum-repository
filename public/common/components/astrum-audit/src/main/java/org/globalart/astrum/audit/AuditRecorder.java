

package org.globalart.astrum.audit;

/**
 * Audit data recorder.
 *
 * @since 3.1
 */
public interface AuditRecorder
{
  boolean isEnabled();

  /**
   * Record audit data and fires {@link AuditDataRecordedEvent}.
   */
  void record(AuditData data);
}
