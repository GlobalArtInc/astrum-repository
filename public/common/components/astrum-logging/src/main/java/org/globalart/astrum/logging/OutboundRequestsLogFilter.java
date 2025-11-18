
package org.globalart.astrum.logging;

import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.filter.Filter;
import ch.qos.logback.core.spi.FilterReply;

import static ch.qos.logback.core.spi.FilterReply.ACCEPT;
import static ch.qos.logback.core.spi.FilterReply.DENY;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.OUTBOUND_REQUESTS_LOG_ONLY;

/**
 * Logback {@link Filter} for outbound requests logs. Ensures that the outbound requests logs get the appropriate
 * entries.
 *
 */
public class OutboundRequestsLogFilter
    extends Filter<ILoggingEvent>
{
  @Override
  public FilterReply decide(final ILoggingEvent event) {
    return OUTBOUND_REQUESTS_LOG_ONLY.equals(event.getMarker()) ? ACCEPT : DENY;
  }
}
