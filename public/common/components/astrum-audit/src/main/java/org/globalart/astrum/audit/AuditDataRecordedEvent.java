

package org.globalart.astrum.audit;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event fired after {@link AuditData} had been recorded.
 *
 * @since 3.1
 */
public class AuditDataRecordedEvent
{
  private final AuditData data;

  public AuditDataRecordedEvent(final AuditData data) {
    this.data = checkNotNull(data);
  }

  public AuditData getData() {
    return data;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "data=" + data +
        '}';
  }
}
