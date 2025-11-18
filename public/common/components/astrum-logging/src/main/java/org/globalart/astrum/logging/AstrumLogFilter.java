
package org.globalart.astrum.logging;

import java.util.Arrays;
import java.util.List;

import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.filter.Filter;
import ch.qos.logback.core.spi.FilterReply;
import org.slf4j.MDC;
import org.slf4j.Marker;

import static ch.qos.logback.core.spi.FilterReply.DENY;
import static ch.qos.logback.core.spi.FilterReply.NEUTRAL;
import static org.globalart.astrum.logging.task.TaskLogger.TASK_LOG_ONLY_MDC;
import static org.globalart.astrum.logging.task.TaskLogger.TASK_LOG_WITH_PROGRESS_MDC;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.AUDIT_LOG_ONLY;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.INTERNAL_PROGRESS;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.OUTBOUND_REQUESTS_LOG_ONLY;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.PROGRESS;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.TASK_LOG_ONLY;

/**
 * Logback {@link Filter} for the main astrum.log
 * - Must NOT have the PROGRESS_LOG marker. These are full progress events for the task log.
 * - Must NOT have the TASK_LOG_ONLY marker. These are task log only events.
 * - Must NOT have TASK_LOG_ONLY_MDC in MDC. These are task log only events.
 *
 * @since 3.5
 */
public class AstrumLogFilter
    extends Filter<ILoggingEvent>
{
  private static final List<Marker> DENY_MARKERS = Arrays.asList(PROGRESS, TASK_LOG_ONLY,
      AUDIT_LOG_ONLY, OUTBOUND_REQUESTS_LOG_ONLY);

  @Override
  public FilterReply decide(final ILoggingEvent event) {
    Marker marker = event.getMarker();

    if (MDC.get(TASK_LOG_WITH_PROGRESS_MDC) != null && INTERNAL_PROGRESS.equals(marker)) {
      // internal progress logs for TaskLogType.TASK_LOG_WITH_PROGRESS are wanted
      return NEUTRAL;
    }

    if (DENY_MARKERS.stream().anyMatch(m -> m.equals(marker)) || MDC.get(TASK_LOG_ONLY_MDC) != null) {
      return DENY;
    }

    return NEUTRAL;
  }
}
