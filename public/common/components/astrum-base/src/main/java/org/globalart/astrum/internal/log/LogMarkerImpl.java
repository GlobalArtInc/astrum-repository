
package org.globalart.astrum.internal.log;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.common.log.LogMarkInsertedEvent;
import org.globalart.astrum.common.log.LogMarker;
import org.globalart.astrum.common.log.LoggerLevel;

import com.google.common.base.Strings;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link LogMarker}.
 * 
 * @since 3.1
 */
@Component
public class LogMarkerImpl
    implements LogMarker
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final LogManager logManager;

  private final EventManager eventManager;

  @Autowired
  public LogMarkerImpl(final LogManager logManager, final EventManager eventManager) {
    this.logManager = checkNotNull(logManager);
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  public void markLog(final String message) {
    // ensure that level for marking logger is enabled
    LoggerLevel loggerLevel = logManager.getLoggerEffectiveLevel(log.getName());
    if (LoggerLevel.INFO.compareTo(loggerLevel) < 0) {
      logManager.setLoggerLevel(log.getName(), LoggerLevel.INFO);
    }

    String asterixes = Strings.repeat("*", message.length() + 4);
    log.info("\n{}\n* {} *\n{}", asterixes, message, asterixes);

    eventManager.post(new LogMarkInsertedEvent(message));
  }
}
