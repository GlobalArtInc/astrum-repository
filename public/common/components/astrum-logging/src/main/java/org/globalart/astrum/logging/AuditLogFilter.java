
package org.globalart.astrum.logging;

import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.filter.Filter;
import ch.qos.logback.core.spi.FilterReply;

import static ch.qos.logback.core.spi.FilterReply.ACCEPT;
import static ch.qos.logback.core.spi.FilterReply.DENY;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.AUDIT_LOG_ONLY;

/**
 * Logback {@link Filter} for audit logs. Ensures that the audit logs get the appropriate entries.
 *
 * @since 3.16
 */
public class AuditLogFilter
    extends Filter<ILoggingEvent>
{
  @Override
  public FilterReply decide(final ILoggingEvent event) {
    return AUDIT_LOG_ONLY.equals(event.getMarker()) ? ACCEPT : DENY;
  }
}
