
package org.globalart.astrum.internal.log;

import org.globalart.astrum.common.log.LoggerLevel;

import ch.qos.logback.classic.Level;

/**
 * Logback levels helper.
 *
 * @since 3.0
 */
public class LogbackLevels
{
  private LogbackLevels() {
    // empty
  }

  /**
   * Convert a Logback {@link Level} into a {@link LoggerLevel}.
   */
  public static LoggerLevel convert(final Level level) {
    switch (level.toInt()) {
      case Level.ERROR_INT:
        return LoggerLevel.ERROR;

      case Level.WARN_INT:
        return LoggerLevel.WARN;

      case Level.INFO_INT:
        return LoggerLevel.INFO;

      case Level.DEBUG_INT:
        return LoggerLevel.DEBUG;

      case Level.OFF_INT:
        return LoggerLevel.OFF;

      case Level.TRACE_INT:
      default:
        return LoggerLevel.TRACE;
    }
  }

  /**
   * Convert a {@link LoggerLevel} into a Logback {@link Level}.
   */
  public static Level convert(final LoggerLevel level) {
    return Level.valueOf(level.name());
  }
}
